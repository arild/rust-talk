package no.posten.parcelapi.controller.response

import com.fasterxml.jackson.annotation.JsonValue

enum class ProductNameResponse(@JsonValue val productName: String) {
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
