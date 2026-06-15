package no.posten.parcelapi.controller

import no.posten.parcelapi.controller.response.BrandingResponse
import no.posten.parcelapi.controller.response.CustomerServiceResponse
import no.posten.parcelapi.controller.response.CustomsInformationRequirements as CustomsInformationRequirementsResponse
import no.posten.parcelapi.controller.response.CustomsPrice
import no.posten.parcelapi.controller.response.CustomsTaxResponse
import no.posten.parcelapi.controller.response.DeliveryResponse
import no.posten.parcelapi.controller.response.DeliveryTimeResponse
import no.posten.parcelapi.controller.response.DeliveryWindowResponse
import no.posten.parcelapi.controller.response.DimensionsResponse
import no.posten.parcelapi.controller.response.DirectionResponse
import no.posten.parcelapi.controller.response.EventResponse
import no.posten.parcelapi.controller.response.FeatureResponse
import no.posten.parcelapi.controller.response.ImageResponse
import no.posten.parcelapi.controller.response.LinkResponse
import no.posten.parcelapi.controller.response.ParcelContent
import no.posten.parcelapi.controller.response.ParcelResponse
import no.posten.parcelapi.controller.response.ParcelStatusResponse
import no.posten.parcelapi.controller.response.ProductGroupResponse
import no.posten.parcelapi.controller.response.ProductNameResponse
import no.posten.parcelapi.controller.response.RecipientResponse
import no.posten.parcelapi.controller.response.RewardsEarningResponse
import no.posten.parcelapi.controller.response.RewardsResponse
import no.posten.parcelapi.controller.response.SenderResponse
import no.posten.parcelapi.controller.response.TransportResponse
import no.posten.parcelapi.service.domain.Branding
import no.posten.parcelapi.service.domain.CustomerService
import no.posten.parcelapi.service.domain.CustomsInformationRequirements
import no.posten.parcelapi.service.domain.CustomsPriceItem
import no.posten.parcelapi.service.domain.CustomsTax
import no.posten.parcelapi.service.domain.Delivery
import no.posten.parcelapi.service.domain.DeliveryTime
import no.posten.parcelapi.service.domain.DeliveryWindow
import no.posten.parcelapi.service.domain.Dimensions
import no.posten.parcelapi.service.domain.Direction
import no.posten.parcelapi.service.domain.Event
import no.posten.parcelapi.service.domain.Feature
import no.posten.parcelapi.service.domain.Image
import no.posten.parcelapi.service.domain.Link
import no.posten.parcelapi.service.domain.Parcel
import no.posten.parcelapi.service.domain.ParcelContentItem
import no.posten.parcelapi.service.domain.ParcelStatus
import no.posten.parcelapi.service.domain.ProductGroup
import no.posten.parcelapi.service.domain.ProductName
import no.posten.parcelapi.service.domain.Recipient
import no.posten.parcelapi.service.domain.Rewards
import no.posten.parcelapi.service.domain.RewardsEarning
import no.posten.parcelapi.service.domain.Sender
import no.posten.parcelapi.service.domain.Transport

fun Parcel.toResponse(): ParcelResponse = ParcelResponse(
    parcelNumber = parcelNumber,
    consignmentNumber = consignmentNumber,
    status = status.toResponse(),
    userChosenName = userChosenName,
    direction = direction.toResponse(),
    transport = transport?.toResponse(),
    dimensions = dimensions?.toResponse(),
    weightInKg = weightInKg,
    delivery = delivery?.toResponse(),
    sender = sender?.toResponse(),
    recipient = recipient?.toResponse(),
    productName = productName.toResponse(),
    productGroup = productGroup.toResponse(),
    events = events.map { it.toResponse() },
    features = features.map { it.toResponse() }.toSet(),
    expiresAt = expiresAt,
    parcelNumbersInConsignment = parcelNumbersInConsignment,
    customsTax = customsTax?.toResponse(),
    customsInformationRequirements = customsInformationRequirements?.toResponse(),
    rewards = rewards?.toResponse(),
)

private fun ParcelStatus.toResponse() = when (this) {
    ParcelStatus.Notified -> ParcelStatusResponse.Notified
    ParcelStatus.Underway -> ParcelStatusResponse.Underway
    ParcelStatus.Collectable -> ParcelStatusResponse.Collectable
    ParcelStatus.ReturnUnderway -> ParcelStatusResponse.ReturnUnderway
    ParcelStatus.ReturnCollectable -> ParcelStatusResponse.ReturnCollectable
    ParcelStatus.Archived -> ParcelStatusResponse.Archived
    ParcelStatus.ArchivedByUser -> ParcelStatusResponse.ArchivedByUser
    ParcelStatus.Unknown -> ParcelStatusResponse.Unknown
}

private fun Direction.toResponse() = when (this) {
    Direction.Sent -> DirectionResponse.Sent
    Direction.Receive -> DirectionResponse.Receive
    Direction.Unknown -> DirectionResponse.Unknown
}

private fun ProductGroup.toResponse() = when (this) {
    ProductGroup.Return -> ProductGroupResponse.Return
    ProductGroup.Mailbox -> ProductGroupResponse.Mailbox
    ProductGroup.PickupPoint -> ProductGroupResponse.PickupPoint
    ProductGroup.HomeDelivery -> ProductGroupResponse.HomeDelivery
    ProductGroup.ParcelLocker -> ProductGroupResponse.ParcelLocker
    ProductGroup.Unknown -> ProductGroupResponse.Unknown
}

private fun ProductName.toResponse() = when (this) {
    ProductName.NorwayParcel -> ProductNameResponse.NorwayParcel
    ProductName.NorwayParcelExpress -> ProductNameResponse.NorwayParcelExpress
    ProductName.NorwayParcelSmall -> ProductNameResponse.NorwayParcelSmall
    ProductName.ParcelPrivateToMailbox -> ProductNameResponse.ParcelPrivateToMailbox
    ProductName.ParcelPrivateToParcelLocker -> ProductNameResponse.ParcelPrivateToParcelLocker
    ProductName.ParcelPrivateToPickupPoint -> ProductNameResponse.ParcelPrivateToPickupPoint
    ProductName.BusinessParcelStandard -> ProductNameResponse.BusinessParcelStandard
    ProductName.BusinessParcelExpressOvernight -> ProductNameResponse.BusinessParcelExpressOvernight
    ProductName.HomeDeliveryParcelReturn -> ProductNameResponse.HomeDeliveryParcelReturn
    ProductName.PickupParcel -> ProductNameResponse.PickupParcel
    ProductName.PickupParcelLocker -> ProductNameResponse.PickupParcelLocker
    ProductName.RegisteredLetter -> ProductNameResponse.RegisteredLetter
    ProductName.BringPack -> ProductNameResponse.BringPack
    ProductName.Unknown -> ProductNameResponse.Unknown
}

private fun Transport.toResponse() = TransportResponse(
    type = type,
    electric = electric,
    fuelType = fuelType,
)

private fun Dimensions.toResponse() = DimensionsResponse(
    lengthInCm = lengthInCm,
    widthInCm = widthInCm,
    heightInCm = heightInCm,
)

private fun Delivery.toResponse() = DeliveryResponse(
    type = type,
    pickUpPointId = pickUpPointId,
    deadlineDate = deadlineDate,
    extendDeadlineTo = extendDeadlineTo,
    pickUpCode = pickUpCode,
    pickUpQrCode = pickUpQrCode,
    shelfNumber = shelfNumber,
    gateCode = gateCode,
    pinCode = pinCode,
    qrCode = qrCode,
    permission = permission,
    bankidAuthenticated = bankidAuthenticated,
    options = options,
    progressPercentageBasedOnEvents = progressPercentageBasedOnEvents,
    deliveryTime = deliveryTime?.toResponse(),
)

private fun DeliveryTime.toResponse() = DeliveryTimeResponse(
    date = date,
    deliveryWindow = deliveryWindow?.toResponse(),
)

private fun DeliveryWindow.toResponse() = DeliveryWindowResponse(
    start = start,
    end = end,
)

private fun Sender.toResponse() = SenderResponse(
    name = name,
    iconUrl = iconUrl,
    phoneNumber = phoneNumber,
    email = email,
    customerNumber = customerNumber,
    branding = branding?.toResponse(),
    customerService = customerService?.toResponse(),
)

private fun Branding.toResponse() = BrandingResponse(
    bannerImage = bannerImage?.toResponse(),
    contentImage = contentImage?.toResponse(),
    title = title,
    description = description,
    storeLink = storeLink?.toResponse(),
)

private fun Image.toResponse() = ImageResponse(url = url, text = text)

private fun Link.toResponse() = LinkResponse(url = url, text = text)

private fun CustomerService.toResponse() = CustomerServiceResponse(name = name, url = url)

private fun Recipient.toResponse() = RecipientResponse(
    name = name,
    postalCode = postalCode,
    city = city,
    address = address,
    phoneNumber = phoneNumber,
    email = email,
    sharedAccessPhoneNumber = sharedAccessPhoneNumber,
    bankidAuthenticated = bankidAuthenticated,
)

private fun Event.toResponse() = EventResponse(
    description = description,
    date = date,
    city = city,
    countryCode = countryCode,
    type = type,
    cause = cause,
    whatsNextDescription = whatsNextDescription,
    displayStatus = displayStatus,
)

private fun Feature.toResponse() = FeatureResponse(
    type = type,
    url = url,
    title = title,
    description = description,
    date = date,
)

private fun CustomsTax.toResponse() = CustomsTaxResponse(
    type = type,
    status = status,
    totalAmountInOre = totalAmountInOre,
    parcelContent = parcelContent.map { it.toResponse() },
    customsPriceList = customsPriceList.map { it.toResponse() },
    dueDate = dueDate,
)

private fun ParcelContentItem.toResponse() = ParcelContent(
    description = description,
    currencyCode = currencyCode,
    amountInSubunit = amountInSubunit,
)

private fun CustomsPriceItem.toResponse() = CustomsPrice(
    description = description,
    amountInOre = amountInOre,
)

private fun CustomsInformationRequirements.toResponse() = CustomsInformationRequirementsResponse(
    documentsRequired = documentsRequired,
    informationProvided = informationProvided,
    identificationRequired = identificationRequired,
)

private fun Rewards.toResponse() = RewardsResponse(
    rewardsEarnings = rewardsEarnings.map { it.toResponse() },
)

private fun RewardsEarning.toResponse() = RewardsEarningResponse(
    validFrom = validFrom,
    validTo = validTo,
    type = type,
    text = text,
    coins = coins,
)
