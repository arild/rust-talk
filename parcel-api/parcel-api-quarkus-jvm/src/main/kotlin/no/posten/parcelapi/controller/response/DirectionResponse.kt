package no.posten.parcelapi.controller.response

import com.fasterxml.jackson.annotation.JsonValue

enum class DirectionResponse(@JsonValue val typeName: String) {
    Sent("sent"),
    Receive("receive"),
    Unknown("unknown"),
}
