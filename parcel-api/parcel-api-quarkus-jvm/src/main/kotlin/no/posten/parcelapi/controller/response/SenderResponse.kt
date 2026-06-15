package no.posten.parcelapi.controller.response

data class SenderResponse(
    val name: String,
    val iconUrl: String?,
    val phoneNumber: String?,
    val email: String?,
    val customerNumber: String?,
    val branding: BrandingResponse?,
    val customerService: CustomerServiceResponse?,
)

data class BrandingResponse(
    val bannerImage: ImageResponse?,
    val contentImage: ImageResponse?,
    val title: String?,
    val description: String?,
    val storeLink: LinkResponse?,
)

data class ImageResponse(val url: String, val text: String?)

data class LinkResponse(val url: String, val text: String?)

data class CustomerServiceResponse(
    val name: String,
    val url: String,
)
