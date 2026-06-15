package no.posten.parcelapi.controller.response

import java.time.Instant

data class EventResponse(
    val description: String,
    val date: Instant,
    val city: String?,
    val countryCode: String?,
    val type: String,
    val cause: String?,
    val whatsNextDescription: String?,
    val displayStatus: String,
)
