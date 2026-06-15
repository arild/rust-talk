package no.posten.parcelapi.controller.response

data class RecipientResponse(
    val name: String?,
    val postalCode: String?,
    val city: String?,
    val address: String?,
    val phoneNumber: String?,
    val email: String?,
    val sharedAccessPhoneNumber: String?,
    val bankidAuthenticated: Boolean,
)
