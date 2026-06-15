package no.posten.parcelapi.service.domain

import java.time.Instant

data class Event(
    val description: String,
    val date: Instant,
    val city: String?,
    val countryCode: String?,
    val type: String,
    val cause: String?,
    val whatsNextDescription: String?,
    val displayStatus: String,
)
