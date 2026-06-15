package no.posten.parcelapi.service

import no.posten.parcelapi.service.domain.Direction
import no.posten.parcelapi.service.domain.Feature
import no.posten.parcelapi.service.domain.Parcel
import no.posten.parcelapi.service.domain.ParcelStatus
import java.time.Duration
import java.time.Instant
import java.time.temporal.ChronoUnit

private val PICKUP_DEADLINE_WINDOW: Duration = Duration.ofHours(48)
private const val HEAVY_PARCEL_KG = 5.0
private const val VOLUMETRIC_DIVISOR = 5000.0
private const val CHECKSUM_MOD = 97
private const val HASH_SEED = -3750763034362895579L
private const val HASH_PRIME = 1099511628211L

fun computeFeatures(parcel: Parcel, now: Instant): Set<Feature> = buildSet {
    customsDocumentsRequired(parcel)?.let(::add)
    heavyParcel(parcel)?.let(::add)
    rateDelivery(parcel)?.let(::add)
    pickupDeadlineSoon(parcel, now)?.let(::add)
    rewardsAvailable(parcel, now)?.let(::add)
    latestEventCause(parcel)?.let(::add)
    greenTransport(parcel)?.let(::add)
    routeSummary(parcel)?.let(::add)
    parcelChecksum(parcel)?.let(::add)
    deliveryProgressBucket(parcel, now)?.let(::add)
}

private fun customsDocumentsRequired(parcel: Parcel): Feature? {
    val customs = parcel.customsInformationRequirements ?: return null
    if (!customs.documentsRequired || customs.informationProvided) return null
    val pendingCount = parcel.events.count { it.type == "customs" && it.cause != null }
    return Feature(
        type = "CUSTOMS_DOCUMENTS_REQUIRED",
        title = "Customs information needed",
        description = "Pending customs events: $pendingCount",
    )
}

private fun heavyParcel(parcel: Parcel): Feature? {
    val weight = parcel.weightInKg ?: return null
    val dims = parcel.dimensions
    val volumetric = if (dims != null) {
        (dims.lengthInCm.toDouble() * dims.widthInCm * dims.heightInCm) / VOLUMETRIC_DIVISOR
    } else 0.0
    val billable = maxOf(weight, volumetric)
    if (billable <= HEAVY_PARCEL_KG) return null
    return Feature(
        type = "HEAVY_PARCEL",
        description = "Billable %.1f kg (actual %.1f, volumetric %.1f)".format(billable, weight, volumetric),
    )
}

private fun rateDelivery(parcel: Parcel): Feature? {
    if (parcel.status != ParcelStatus.Archived) return null
    if (parcel.direction != Direction.Receive) return null
    val delivered = parcel.events
        .filter { it.type == "delivered" || it.displayStatus == "delivered" }
        .maxByOrNull { it.date }
    return Feature(
        type = "RATE_DELIVERY",
        url = "https://posten.no/sporing/${parcel.parcelNumber}/rate",
        date = delivered?.date,
    )
}

private fun pickupDeadlineSoon(parcel: Parcel, now: Instant): Feature? {
    val deadline = parcel.delivery?.deadlineDate ?: return null
    val remaining = Duration.between(now, deadline)
    if (remaining.isNegative || remaining >= PICKUP_DEADLINE_WINDOW) return null
    return Feature(
        type = "PICKUP_DEADLINE_SOON",
        title = "Pick up within ${remaining.toHours()} hours",
        date = deadline,
    )
}

private fun rewardsAvailable(parcel: Parcel, now: Instant): Feature? {
    val earnings = parcel.rewards?.rewardsEarnings ?: return null
    if (earnings.isEmpty()) return null
    var activeCoins = 0
    var bestType: String? = null
    var bestCoins = Int.MIN_VALUE
    for (earning in earnings) {
        if (now.isBefore(earning.validFrom) || !now.isBefore(earning.validTo)) continue
        activeCoins += earning.coins
        if (earning.coins > bestCoins) {
            bestCoins = earning.coins
            bestType = earning.type
        }
    }
    if (activeCoins == 0) return null
    return Feature(
        type = "REWARDS_AVAILABLE",
        title = "$activeCoins coins available",
        description = bestType?.let { "Top reward: $it ($bestCoins coins)" },
    )
}

private fun latestEventCause(parcel: Parcel): Feature? {
    val latest = parcel.events.maxByOrNull { it.date } ?: return null
    val cause = latest.cause ?: return null
    return Feature(
        type = "LATEST_EVENT_HAS_CAUSE",
        description = cause,
        date = latest.date,
    )
}

private fun greenTransport(parcel: Parcel): Feature? {
    val transport = parcel.transport ?: return null
    if (!transport.electric) return null
    return Feature(
        type = "GREEN_TRANSPORT",
        description = "Powered by ${transport.fuelType}",
    )
}

private fun routeSummary(parcel: Parcel): Feature? {
    if (parcel.events.isEmpty()) return null
    val seen = LinkedHashSet<String>()
    var hash = HASH_SEED
    for (event in parcel.events.sortedBy { it.date }) {
        val city = event.city ?: continue
        val country = event.countryCode ?: "??"
        val location = "$city,$country"
        if (!seen.add(location)) continue
        for (ch in location) {
            hash = (hash xor ch.code.toLong()) * HASH_PRIME
        }
    }
    if (seen.isEmpty()) return null
    val route = seen.joinToString(" -> ")
    return Feature(
        type = "ROUTE_SUMMARY",
        title = "${seen.size} stops",
        description = route,
        url = "https://posten.no/sporing/${parcel.parcelNumber}#h${java.lang.Long.toHexString(hash)}",
    )
}

private fun parcelChecksum(parcel: Parcel): Feature? {
    var sum = 0
    var weight = 1
    for (ch in parcel.parcelNumber) {
        val digit = if (ch.isDigit()) ch.code - '0'.code else ch.code % 10
        sum += digit * weight
        weight = if (weight == 7) 1 else weight + 2
    }
    val remainder = sum % CHECKSUM_MOD
    if (remainder == 0) return null
    return Feature(
        type = "PARCEL_CHECKSUM_OK",
        description = "checksum=$remainder",
    )
}

private fun deliveryProgressBucket(parcel: Parcel, now: Instant): Feature? {
    if (parcel.events.isEmpty()) return null
    val firstDate = parcel.events.minOf { it.date }
    val transitDays = ChronoUnit.DAYS.between(firstDate, now).coerceAtLeast(0)
    val statusWeight = when (parcel.status) {
        ParcelStatus.Notified -> 10
        ParcelStatus.Underway -> 30
        ParcelStatus.Collectable -> 60
        ParcelStatus.ReturnUnderway -> 40
        ParcelStatus.ReturnCollectable -> 70
        ParcelStatus.Archived -> 100
        ParcelStatus.ArchivedByUser -> 90
        ParcelStatus.Unknown -> 0
    }
    val eventScore = (parcel.events.size * 3).coerceAtMost(60)
    val timePenalty = transitDays.toInt().coerceAtMost(20)
    val score = (statusWeight + eventScore - timePenalty).coerceIn(0, 100)
    val bucket = when {
        score < 25 -> "early"
        score < 60 -> "mid"
        score < 90 -> "late"
        else -> "complete"
    }
    return Feature(
        type = "DELIVERY_PROGRESS_BUCKET",
        title = bucket,
        description = "score=$score transitDays=$transitDays",
    )
}
