use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

/// Java `Instant.toString()`-compatible serializer.
/// Omits fractional seconds when zero, otherwise emits 3/6/9 digits.
pub mod instant_format {
    use chrono::{DateTime, Timelike, Utc};
    use serde::Serializer;

    pub fn serialize<S: Serializer>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format_instant(dt))
    }

    pub(super) fn format_instant(dt: &DateTime<Utc>) -> String {
        let nanos = dt.nanosecond();
        let base = dt.format("%Y-%m-%dT%H:%M:%S");
        if nanos == 0 {
            format!("{base}Z")
        } else if nanos % 1_000_000 == 0 {
            format!("{base}.{:03}Z", nanos / 1_000_000)
        } else if nanos % 1_000 == 0 {
            format!("{base}.{:06}Z", nanos / 1_000)
        } else {
            format!("{base}.{:09}Z", nanos)
        }
    }
}

pub mod instant_format_opt {
    use super::instant_format::format_instant;
    use chrono::{DateTime, Utc};
    use serde::Serializer;

    pub fn serialize<S: Serializer>(
        dt: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match dt {
            Some(value) => serializer.serialize_str(&format_instant(value)),
            None => serializer.serialize_none(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParcelResponse {
    pub parcel_number: String,
    pub consignment_number: String,
    pub status: ParcelStatusResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_chosen_name: Option<String>,
    pub direction: DirectionResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<DimensionsResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_in_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<DeliveryResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<SenderResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<RecipientResponse>,
    pub product_name: ProductNameResponse,
    pub product_group: ProductGroupResponse,
    pub events: Vec<EventResponse>,
    pub features: Vec<FeatureResponse>,
    #[serde(with = "instant_format")]
    pub expires_at: DateTime<Utc>,
    pub parcel_numbers_in_consignment: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customs_tax: Option<CustomsTaxResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customs_information_requirements: Option<CustomsInformationRequirements>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards: Option<RewardsResponse>,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParcelStatusResponse {
    Notified,
    Underway,
    Collectable,
    ReturnUnderway,
    ReturnCollectable,
    Archived,
    ArchivedByUser,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DirectionResponse {
    Sent,
    Receive,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProductGroupResponse {
    Return,
    Mailbox,
    PickupPoint,
    HomeDelivery,
    ParcelLocker,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProductNameResponse {
    NorwayParcel,
    NorwayParcelExpress,
    NorwayParcelSmall,
    ParcelPrivateToMailbox,
    ParcelPrivateToParcelLocker,
    ParcelPrivateToPickupPoint,
    BusinessParcelStandard,
    BusinessParcelExpressOvernight,
    HomeDeliveryParcelReturn,
    PickupParcel,
    PickupParcelLocker,
    RegisteredLetter,
    BringPack,
    Unknown,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransportResponse {
    pub r#type: String,
    pub electric: bool,
    pub fuel_type: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DimensionsResponse {
    pub length_in_cm: i32,
    pub width_in_cm: i32,
    pub height_in_cm: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryResponse {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pick_up_point_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", with = "instant_format_opt")]
    pub deadline_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", with = "instant_format_opt")]
    pub extend_deadline_to: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pick_up_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pick_up_qr_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shelf_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_code: Option<String>,
    pub permission: String,
    pub bankid_authenticated: String,
    pub options: Vec<String>,
    pub progress_percentage_based_on_events: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_time: Option<DeliveryTimeResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryTimeResponse {
    #[serde(with = "instant_format")]
    pub date: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_window: Option<DeliveryWindowResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryWindowResponse {
    #[serde(with = "instant_format")]
    pub start: DateTime<Utc>,
    #[serde(with = "instant_format")]
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SenderResponse {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding: Option<BrandingResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_service: Option<CustomerServiceResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrandingResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_image: Option<ImageResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_image: Option<ImageResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_link: Option<LinkResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkResponse {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomerServiceResponse {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecipientResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_access_phone_number: Option<String>,
    pub bankid_authenticated: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventResponse {
    pub description: String,
    #[serde(with = "instant_format")]
    pub date: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whats_next_description: Option<String>,
    pub display_status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureResponse {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", with = "instant_format_opt")]
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomsTaxResponse {
    pub r#type: String,
    pub status: String,
    pub total_amount_in_ore: i32,
    pub parcel_content: Vec<ParcelContent>,
    pub customs_price_list: Vec<CustomsPrice>,
    #[serde(skip_serializing_if = "Option::is_none", with = "instant_format_opt")]
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParcelContent {
    pub description: String,
    pub currency_code: String,
    pub amount_in_subunit: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomsPrice {
    pub description: String,
    pub amount_in_ore: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomsInformationRequirements {
    pub documents_required: bool,
    pub information_provided: bool,
    pub identification_required: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RewardsResponse {
    pub rewards_earnings: Vec<RewardsEarningResponse>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RewardsEarningResponse {
    #[serde(with = "instant_format")]
    pub valid_from: DateTime<Utc>,
    #[serde(with = "instant_format")]
    pub valid_to: DateTime<Utc>,
    pub r#type: String,
    pub text: String,
    pub coins: i32,
}
