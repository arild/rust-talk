package no.posten.parcelapi.service.domain

import java.time.Instant

data class Feature(
    val type: String,
    val url: String? = null,
    val title: String? = null,
    val description: String? = null,
    val date: Instant? = null,
)
