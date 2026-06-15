package no.posten.parcelapi.controller.response

import java.time.Instant

data class DeliveryResponse(
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
    val deliveryTime: DeliveryTimeResponse?,
)

data class DeliveryTimeResponse(
    val date: Instant,
    val deliveryWindow: DeliveryWindowResponse?,
)

data class DeliveryWindowResponse(
    val start: Instant,
    val end: Instant,
)
