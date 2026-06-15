package no.posten.parcelapi.service.domain

import java.time.Instant

data class Delivery(
    val type: String,
    val pickUpPointId: String?,
    val deadlineDate: Instant?,
    val extendDeadlineTo: Instant?,
    val pickUpCode: String?,
    val pickUpQrCode: String?,
    val shelfNumber: String?,
    val gateCode: String?,
    val pinCode: String?,
    val qrCode: String?,
    val permission: String,
    val bankidAuthenticated: String,
    val options: List<String>,
    val progressPercentageBasedOnEvents: Int,
    val deliveryTime: DeliveryTime?,
)

data class DeliveryTime(
    val date: Instant,
    val deliveryWindow: DeliveryWindow?,
)

data class DeliveryWindow(
    val start: Instant,
    val end: Instant,
)
