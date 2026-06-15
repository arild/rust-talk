use chrono::{DateTime, Duration, Utc};

use crate::domain::parcel::{Direction, Feature, Parcel, ParcelStatus};

const PICKUP_DEADLINE_WINDOW_HOURS: i64 = 48;
const HEAVY_PARCEL_KG: f64 = 5.0;
const VOLUMETRIC_DIVISOR: f64 = 5000.0;
const CHECKSUM_MOD: i32 = 97;
const HASH_SEED: u64 = 0xcbf29ce484222325;
const HASH_PRIME: u64 = 0x100000001b3;

pub fn compute_features(parcel: &Parcel, now: DateTime<Utc>) -> Vec<Feature> {
    let mut out: Vec<Feature> = Vec::with_capacity(10);
    if let Some(f) = customs_documents_required(parcel) {
        out.push(f);
    }
    if let Some(f) = heavy_parcel(parcel) {
        out.push(f);
    }
    if let Some(f) = rate_delivery(parcel) {
        out.push(f);
    }
    if let Some(f) = pickup_deadline_soon(parcel, now) {
        out.push(f);
    }
    if let Some(f) = rewards_available(parcel, now) {
        out.push(f);
    }
    if let Some(f) = latest_event_cause(parcel) {
        out.push(f);
    }
    if let Some(f) = green_transport(parcel) {
        out.push(f);
    }
    if let Some(f) = route_summary(parcel) {
        out.push(f);
    }
    if let Some(f) = parcel_checksum(parcel) {
        out.push(f);
    }
    if let Some(f) = delivery_progress_bucket(parcel, now) {
        out.push(f);
    }
    out
}

fn customs_documents_required(parcel: &Parcel) -> Option<Feature> {
    let customs = parcel.customs_information_requirements.as_ref()?;
    if !customs.documents_required || customs.information_provided {
        return None;
    }
    let pending_count = parcel
        .events
        .iter()
        .filter(|e| e.r#type == "customs" && e.cause.is_some())
        .count();
    Some(Feature {
        r#type: "CUSTOMS_DOCUMENTS_REQUIRED".to_string(),
        url: None,
        title: Some("Customs information needed".to_string()),
        description: Some(format!("Pending customs events: {pending_count}")),
        date: None,
    })
}

fn heavy_parcel(parcel: &Parcel) -> Option<Feature> {
    let weight = parcel.weight_in_kg?;
    let volumetric = match &parcel.dimensions {
        Some(d) => {
            (d.length_in_cm as f64 * d.width_in_cm as f64 * d.height_in_cm as f64)
                / VOLUMETRIC_DIVISOR
        }
        None => 0.0,
    };
    let billable = weight.max(volumetric);
    if billable <= HEAVY_PARCEL_KG {
        return None;
    }
    Some(Feature {
        r#type: "HEAVY_PARCEL".to_string(),
        url: None,
        title: None,
        description: Some(format!(
            "Billable {billable:.1} kg (actual {weight:.1}, volumetric {volumetric:.1})"
        )),
        date: None,
    })
}

fn rate_delivery(parcel: &Parcel) -> Option<Feature> {
    if !matches!(parcel.status, ParcelStatus::Archived) {
        return None;
    }
    if !matches!(parcel.direction, Direction::Receive) {
        return None;
    }
    let delivered_date = parcel
        .events
        .iter()
        .filter(|e| e.r#type == "delivered" || e.display_status == "delivered")
        .map(|e| e.date)
        .max();
    Some(Feature {
        r#type: "RATE_DELIVERY".to_string(),
        url: Some(format!(
            "https://posten.no/sporing/{}/rate",
            parcel.parcel_number
        )),
        title: None,
        description: None,
        date: delivered_date,
    })
}

fn pickup_deadline_soon(parcel: &Parcel, now: DateTime<Utc>) -> Option<Feature> {
    let deadline = parcel.delivery.as_ref()?.deadline_date?;
    let remaining = deadline - now;
    if remaining < Duration::zero() || remaining >= Duration::hours(PICKUP_DEADLINE_WINDOW_HOURS) {
        return None;
    }
    Some(Feature {
        r#type: "PICKUP_DEADLINE_SOON".to_string(),
        url: None,
        title: Some(format!("Pick up within {} hours", remaining.num_hours())),
        description: None,
        date: Some(deadline),
    })
}

fn rewards_available(parcel: &Parcel, now: DateTime<Utc>) -> Option<Feature> {
    let earnings = &parcel.rewards.as_ref()?.rewards_earnings;
    if earnings.is_empty() {
        return None;
    }
    let mut active_coins: i32 = 0;
    let mut best_type: Option<&str> = None;
    let mut best_coins = i32::MIN;
    for earning in earnings {
        if now < earning.valid_from || now >= earning.valid_to {
            continue;
        }
        active_coins += earning.coins;
        if earning.coins > best_coins {
            best_coins = earning.coins;
            best_type = Some(&earning.r#type);
        }
    }
    if active_coins == 0 {
        return None;
    }
    Some(Feature {
        r#type: "REWARDS_AVAILABLE".to_string(),
        url: None,
        title: Some(format!("{active_coins} coins available")),
        description: best_type.map(|t| format!("Top reward: {t} ({best_coins} coins)")),
        date: None,
    })
}

fn latest_event_cause(parcel: &Parcel) -> Option<Feature> {
    let latest = parcel.events.iter().max_by_key(|e| e.date)?;
    let cause = latest.cause.as_ref()?;
    Some(Feature {
        r#type: "LATEST_EVENT_HAS_CAUSE".to_string(),
        url: None,
        title: None,
        description: Some(cause.clone()),
        date: Some(latest.date),
    })
}

fn green_transport(parcel: &Parcel) -> Option<Feature> {
    let transport = parcel.transport.as_ref()?;
    if !transport.electric {
        return None;
    }
    Some(Feature {
        r#type: "GREEN_TRANSPORT".to_string(),
        url: None,
        title: None,
        description: Some(format!("Powered by {}", transport.fuel_type)),
        date: None,
    })
}

fn route_summary(parcel: &Parcel) -> Option<Feature> {
    if parcel.events.is_empty() {
        return None;
    }
    let mut by_date: Vec<_> = parcel.events.iter().collect();
    by_date.sort_by_key(|e| e.date);
    let mut seen: Vec<String> = Vec::new();
    let mut hash: u64 = HASH_SEED;
    for event in by_date {
        let Some(city) = event.city.as_deref() else {
            continue;
        };
        let country = event.country_code.as_deref().unwrap_or("??");
        let location = format!("{city},{country}");
        if seen.iter().any(|s| s == &location) {
            continue;
        }
        for ch in location.chars() {
            hash = (hash ^ (ch as u64)).wrapping_mul(HASH_PRIME);
        }
        seen.push(location);
    }
    if seen.is_empty() {
        return None;
    }
    let count = seen.len();
    let route = seen.join(" -> ");
    Some(Feature {
        r#type: "ROUTE_SUMMARY".to_string(),
        url: Some(format!(
            "https://posten.no/sporing/{}#h{hash:x}",
            parcel.parcel_number
        )),
        title: Some(format!("{count} stops")),
        description: Some(route),
        date: None,
    })
}

fn parcel_checksum(parcel: &Parcel) -> Option<Feature> {
    let mut sum: i32 = 0;
    let mut weight: i32 = 1;
    for ch in parcel.parcel_number.chars() {
        let digit = if ch.is_ascii_digit() {
            (ch as i32) - ('0' as i32)
        } else {
            (ch as i32) % 10
        };
        sum += digit * weight;
        weight = if weight == 7 { 1 } else { weight + 2 };
    }
    let remainder = sum % CHECKSUM_MOD;
    if remainder == 0 {
        return None;
    }
    Some(Feature {
        r#type: "PARCEL_CHECKSUM_OK".to_string(),
        url: None,
        title: None,
        description: Some(format!("checksum={remainder}")),
        date: None,
    })
}

fn delivery_progress_bucket(parcel: &Parcel, now: DateTime<Utc>) -> Option<Feature> {
    if parcel.events.is_empty() {
        return None;
    }
    let first_date = parcel.events.iter().map(|e| e.date).min()?;
    let transit_days = (now - first_date).num_days().max(0);
    let status_weight = match parcel.status {
        ParcelStatus::Notified => 10,
        ParcelStatus::Underway => 30,
        ParcelStatus::Collectable => 60,
        ParcelStatus::ReturnUnderway => 40,
        ParcelStatus::ReturnCollectable => 70,
        ParcelStatus::Archived => 100,
        ParcelStatus::ArchivedByUser => 90,
        ParcelStatus::Unknown => 0,
    };
    let event_score = (parcel.events.len() as i32 * 3).min(60);
    let time_penalty = (transit_days as i32).min(20);
    let score = (status_weight + event_score - time_penalty).clamp(0, 100);
    let bucket = if score < 25 {
        "early"
    } else if score < 60 {
        "mid"
    } else if score < 90 {
        "late"
    } else {
        "complete"
    };
    Some(Feature {
        r#type: "DELIVERY_PROGRESS_BUCKET".to_string(),
        url: None,
        title: Some(bucket.to_string()),
        description: Some(format!("score={score} transitDays={transit_days}")),
        date: None,
    })
}
