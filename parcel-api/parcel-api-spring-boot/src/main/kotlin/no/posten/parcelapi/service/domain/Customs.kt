package no.posten.parcelapi.service.domain

import java.time.Instant

data class CustomsTax(
    val type: String,
    val status: String,
    val totalAmountInOre: Int,
    val parcelContent: List<ParcelContentItem>,
    val customsPriceList: List<CustomsPriceItem>,
    val dueDate: Instant?,
)

data class ParcelContentItem(
    val description: String,
    val currencyCode: String,
    val amountInSubunit: Int,
)

data class CustomsPriceItem(
    val description: String,
    val amountInOre: Int,
)

data class CustomsInformationRequirements(
    val documentsRequired: Boolean,
    val informationProvided: Boolean,
    val identificationRequired: Boolean,
)
