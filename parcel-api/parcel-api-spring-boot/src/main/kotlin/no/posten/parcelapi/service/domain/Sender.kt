package no.posten.parcelapi.service.domain

data class Sender(
    val name: String,
    val iconUrl: String?,
    val phoneNumber: String?,
    val email: String?,
    val customerNumber: String?,
    val branding: Branding?,
    val customerService: CustomerService?,
)

data class Branding(
    val bannerImage: Image?,
    val contentImage: Image?,
    val title: String?,
    val description: String?,
    val storeLink: Link?,
)

data class Image(val url: String, val text: String?)

data class Link(val url: String, val text: String?)

data class CustomerService(
    val name: String,
    val url: String,
)
