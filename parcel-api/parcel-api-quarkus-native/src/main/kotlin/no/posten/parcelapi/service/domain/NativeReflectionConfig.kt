package no.posten.parcelapi.service.domain

import io.quarkus.runtime.annotations.RegisterForReflection

// StubParcelService deserialises these domain types through a hand-built ObjectMapper,
// so they never surface in a JAX-RS signature and Quarkus can't discover them for
// native-image reflection on its own. Registering them keeps `-Dquarkus.native.enabled`
// builds serving real data; the annotation is inert on the JVM.
@RegisterForReflection(
    targets = [
        Parcel::class,
        ParcelStatus::class,
        Direction::class,
        ProductGroup::class,
        ProductName::class,
        Transport::class,
        Dimensions::class,
        Delivery::class,
        DeliveryTime::class,
        DeliveryWindow::class,
        Sender::class,
        Branding::class,
        Image::class,
        Link::class,
        CustomerService::class,
        Recipient::class,
        Event::class,
        Feature::class,
        CustomsTax::class,
        ParcelContentItem::class,
        CustomsPriceItem::class,
        CustomsInformationRequirements::class,
        Rewards::class,
        RewardsEarning::class,
    ],
)
class NativeReflectionConfig
