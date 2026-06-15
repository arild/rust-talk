package no.posten.parcelapi.controller.response

import com.fasterxml.jackson.annotation.JsonValue

enum class ProductGroupResponse(@JsonValue val typeName: String) {
    Return("return"),
    Mailbox("mailbox"),
    PickupPoint("pickup_point"),
    HomeDelivery("home_delivery"),
    ParcelLocker("parcel_locker"),
    Unknown("unknown"),
}
