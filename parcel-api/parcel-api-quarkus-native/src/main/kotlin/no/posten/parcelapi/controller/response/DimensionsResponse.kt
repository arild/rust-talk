package no.posten.parcelapi.controller.response

data class DimensionsResponse(
    val lengthInCm: Int,
    val widthInCm: Int,
    val heightInCm: Int,
)
