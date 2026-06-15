package no.posten.parcelapi.controller.response

data class TransportResponse(
    val type: String,
    val electric: Boolean,
    val fuelType: String,
)
