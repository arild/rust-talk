package no.posten.parcelapi.controller.response

data class CustomsInformationRequirements(
    val documentsRequired: Boolean,
    val informationProvided: Boolean,
    val identificationRequired: Boolean,
)
