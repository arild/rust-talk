package no.posten.parcelapi.service.domain

import com.fasterxml.jackson.annotation.JsonValue
import java.time.Instant

data class Parcel(
    val parcelNumber: String,
    val consignmentNumber: String,
    val status: ParcelStatus,
    val userChosenName: String?,
    val direction: Direction,
    val transport: Transport?,
    val dimensions: Dimensions?,
    val weightInKg: Double?,
    val delivery: Delivery?,
    val sender: Sender?,
    val recipient: Recipient?,
    val productName: ProductName,
    val productGroup: ProductGroup,
    val events: List<Event>,
    val features: Set<Feature>,
    val expiresAt: Instant,
    val parcelNumbersInConsignment: Set<String>,
    val customsTax: CustomsTax?,
    val customsInformationRequirements: CustomsInformationRequirements?,
    val rewards: Rewards?,
)

enum class ParcelStatus(@JsonValue val typeName: String) {
    Notified("notified"),
    Underway("underway"),
    Collectable("collectable"),
    ReturnUnderway("return_underway"),
    ReturnCollectable("return_collectable"),
    Archived("archived"),
    ArchivedByUser("archived_by_user"),
    Unknown("unknown"),
}

enum class Direction(@JsonValue val typeName: String) {
    Sent("sent"),
    Receive("receive"),
    Unknown("unknown"),
}

enum class ProductGroup(@JsonValue val typeName: String) {
    Return("return"),
    Mailbox("mailbox"),
    PickupPoint("pickup_point"),
    HomeDelivery("home_delivery"),
    ParcelLocker("parcel_locker"),
    Unknown("unknown"),
}

enum class ProductName(@JsonValue val productName: String) {
    NorwayParcel("norway_parcel"),
    NorwayParcelExpress("norway_parcel_express"),
    NorwayParcelSmall("norway_parcel_small"),
    ParcelPrivateToMailbox("parcel_private_to_mailbox"),
    ParcelPrivateToParcelLocker("parcel_private_to_parcel_locker"),
    ParcelPrivateToPickupPoint("parcel_private_to_pickup_point"),
    BusinessParcelStandard("business_parcel_standard"),
    BusinessParcelExpressOvernight("business_parcel_express_overnight"),
    HomeDeliveryParcelReturn("home_delivery_parcel_return"),
    PickupParcel("pickup_parcel"),
    PickupParcelLocker("pickup_parcel_locker"),
    RegisteredLetter("registered_letter"),
    BringPack("bring_pack"),
    Unknown("unknown"),
}

data class Transport(
    val type: String,
    val electric: Boolean,
    val fuelType: String,
)

data class Dimensions(
    val lengthInCm: Int,
    val widthInCm: Int,
    val heightInCm: Int,
)
