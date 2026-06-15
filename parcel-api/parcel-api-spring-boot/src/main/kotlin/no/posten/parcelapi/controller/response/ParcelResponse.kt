package no.posten.parcelapi.controller.response

import java.time.Instant

data class ParcelResponse(
    val parcelNumber: String,
    val consignmentNumber: String,
    val status: ParcelStatusResponse,
    val userChosenName: String?,
    val direction: DirectionResponse,
    val transport: TransportResponse?,
    val dimensions: DimensionsResponse?,
    val weightInKg: Double?,
    val delivery: DeliveryResponse?,
    val sender: SenderResponse?,
    val recipient: RecipientResponse?,
    val productName: ProductNameResponse,
    val productGroup: ProductGroupResponse,
    val events: List<EventResponse>,
    val features: Set<FeatureResponse>,
    val expiresAt: Instant,
    val parcelNumbersInConsignment: Set<String>,
    val customsTax: CustomsTaxResponse?,
    val customsInformationRequirements: CustomsInformationRequirements?,
    val rewards: RewardsResponse?,
)
