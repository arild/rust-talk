package no.posten.parcelapi.controller.response

import java.time.Instant

data class CustomsTaxResponse(
    val type: String,
    val status: String,
    val totalAmountInOre: Int,
    val parcelContent: List<ParcelContent>,
    val customsPriceList: List<CustomsPrice>,
    val dueDate: Instant?,
)

data class ParcelContent(
    val description: String,
    val currencyCode: String,
    val amountInSubunit: Int,
)

data class CustomsPrice(
    val description: String,
    val amountInOre: Int,
)
