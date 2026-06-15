package no.posten.parcelapi.controller.response

import com.fasterxml.jackson.annotation.JsonValue

enum class ParcelStatusResponse(@JsonValue val typeName: String) {
    Notified("notified"),
    Underway("underway"),
    Collectable("collectable"),
    ReturnUnderway("return_underway"),
    ReturnCollectable("return_collectable"),
    Archived("archived"),
    ArchivedByUser("archived_by_user"),
    Unknown("unknown"),
}
