package no.posten.parcelapi.controller.response

import java.time.Instant

data class FeatureResponse(
    val type: String,
    val url: String? = null,
    val title: String? = null,
    val description: String? = null,
    val date: Instant? = null,
)
