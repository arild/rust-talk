use crate::controller::response::{
    BrandingResponse, CustomerServiceResponse, CustomsInformationRequirements, CustomsPrice,
    CustomsTaxResponse, DeliveryResponse, DeliveryTimeResponse, DeliveryWindowResponse,
    DimensionsResponse, DirectionResponse, EventResponse, FeatureResponse, ImageResponse,
    LinkResponse, ParcelContent, ParcelResponse, ParcelStatusResponse,
    ProductGroupResponse, ProductNameResponse, RecipientResponse, RewardsEarningResponse,
    RewardsResponse, SenderResponse, TransportResponse,
};
use crate::domain::parcel as d;

impl From<d::Parcel> for ParcelResponse {
    fn from(v: d::Parcel) -> Self {
        Self {
            parcel_number: v.parcel_number,
            consignment_number: v.consignment_number,
            status: v.status.into(),
            user_chosen_name: v.user_chosen_name,
            direction: v.direction.into(),
            transport: v.transport.map(Into::into),
            dimensions: v.dimensions.map(Into::into),
            weight_in_kg: v.weight_in_kg,
            delivery: v.delivery.map(Into::into),
            sender: v.sender.map(Into::into),
            recipient: v.recipient.map(Into::into),
            product_name: v.product_name.into(),
            product_group: v.product_group.into(),
            events: v.events.into_iter().map(Into::into).collect(),
            features: v.features.into_iter().map(Into::into).collect(),
            expires_at: v.expires_at,
            parcel_numbers_in_consignment: v.parcel_numbers_in_consignment,
            customs_tax: v.customs_tax.map(Into::into),
            customs_information_requirements: v.customs_information_requirements.map(Into::into),
            rewards: v.rewards.map(Into::into),
        }
    }
}

impl From<d::ParcelStatus> for ParcelStatusResponse {
    fn from(v: d::ParcelStatus) -> Self {
        match v {
            d::ParcelStatus::Notified => Self::Notified,
            d::ParcelStatus::Underway => Self::Underway,
            d::ParcelStatus::Collectable => Self::Collectable,
            d::ParcelStatus::ReturnUnderway => Self::ReturnUnderway,
            d::ParcelStatus::ReturnCollectable => Self::ReturnCollectable,
            d::ParcelStatus::Archived => Self::Archived,
            d::ParcelStatus::ArchivedByUser => Self::ArchivedByUser,
            d::ParcelStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<d::Direction> for DirectionResponse {
    fn from(v: d::Direction) -> Self {
        match v {
            d::Direction::Sent => Self::Sent,
            d::Direction::Receive => Self::Receive,
            d::Direction::Unknown => Self::Unknown,
        }
    }
}

impl From<d::ProductGroup> for ProductGroupResponse {
    fn from(v: d::ProductGroup) -> Self {
        match v {
            d::ProductGroup::Return => Self::Return,
            d::ProductGroup::Mailbox => Self::Mailbox,
            d::ProductGroup::PickupPoint => Self::PickupPoint,
            d::ProductGroup::HomeDelivery => Self::HomeDelivery,
            d::ProductGroup::ParcelLocker => Self::ParcelLocker,
            d::ProductGroup::Unknown => Self::Unknown,
        }
    }
}

impl From<d::ProductName> for ProductNameResponse {
    fn from(v: d::ProductName) -> Self {
        match v {
            d::ProductName::NorwayParcel => Self::NorwayParcel,
            d::ProductName::NorwayParcelExpress => Self::NorwayParcelExpress,
            d::ProductName::NorwayParcelSmall => Self::NorwayParcelSmall,
            d::ProductName::ParcelPrivateToMailbox => Self::ParcelPrivateToMailbox,
            d::ProductName::ParcelPrivateToParcelLocker => Self::ParcelPrivateToParcelLocker,
            d::ProductName::ParcelPrivateToPickupPoint => Self::ParcelPrivateToPickupPoint,
            d::ProductName::BusinessParcelStandard => Self::BusinessParcelStandard,
            d::ProductName::BusinessParcelExpressOvernight => Self::BusinessParcelExpressOvernight,
            d::ProductName::HomeDeliveryParcelReturn => Self::HomeDeliveryParcelReturn,
            d::ProductName::PickupParcel => Self::PickupParcel,
            d::ProductName::PickupParcelLocker => Self::PickupParcelLocker,
            d::ProductName::RegisteredLetter => Self::RegisteredLetter,
            d::ProductName::BringPack => Self::BringPack,
            d::ProductName::Unknown => Self::Unknown,
        }
    }
}

impl From<d::Transport> for TransportResponse {
    fn from(v: d::Transport) -> Self {
        Self {
            r#type: v.r#type,
            electric: v.electric,
            fuel_type: v.fuel_type,
        }
    }
}

impl From<d::Dimensions> for DimensionsResponse {
    fn from(v: d::Dimensions) -> Self {
        Self {
            length_in_cm: v.length_in_cm,
            width_in_cm: v.width_in_cm,
            height_in_cm: v.height_in_cm,
        }
    }
}

impl From<d::Delivery> for DeliveryResponse {
    fn from(v: d::Delivery) -> Self {
        Self {
            r#type: v.r#type,
            pick_up_point_id: v.pick_up_point_id,
            deadline_date: v.deadline_date,
            extend_deadline_to: v.extend_deadline_to,
            pick_up_code: v.pick_up_code,
            pick_up_qr_code: v.pick_up_qr_code,
            shelf_number: v.shelf_number,
            gate_code: v.gate_code,
            pin_code: v.pin_code,
            qr_code: v.qr_code,
            permission: v.permission,
            bankid_authenticated: v.bankid_authenticated,
            options: v.options,
            progress_percentage_based_on_events: v.progress_percentage_based_on_events,
            delivery_time: v.delivery_time.map(Into::into),
        }
    }
}

impl From<d::DeliveryTime> for DeliveryTimeResponse {
    fn from(v: d::DeliveryTime) -> Self {
        Self {
            date: v.date,
            delivery_window: v.delivery_window.map(Into::into),
        }
    }
}

impl From<d::DeliveryWindow> for DeliveryWindowResponse {
    fn from(v: d::DeliveryWindow) -> Self {
        Self {
            start: v.start,
            end: v.end,
        }
    }
}

impl From<d::Sender> for SenderResponse {
    fn from(v: d::Sender) -> Self {
        Self {
            name: v.name,
            icon_url: v.icon_url,
            phone_number: v.phone_number,
            email: v.email,
            customer_number: v.customer_number,
            branding: v.branding.map(Into::into),
            customer_service: v.customer_service.map(Into::into),
        }
    }
}

impl From<d::Branding> for BrandingResponse {
    fn from(v: d::Branding) -> Self {
        Self {
            banner_image: v.banner_image.map(Into::into),
            content_image: v.content_image.map(Into::into),
            title: v.title,
            description: v.description,
            store_link: v.store_link.map(Into::into),
        }
    }
}

impl From<d::Image> for ImageResponse {
    fn from(v: d::Image) -> Self {
        Self {
            url: v.url,
            text: v.text,
        }
    }
}

impl From<d::Link> for LinkResponse {
    fn from(v: d::Link) -> Self {
        Self {
            url: v.url,
            text: v.text,
        }
    }
}

impl From<d::CustomerService> for CustomerServiceResponse {
    fn from(v: d::CustomerService) -> Self {
        Self {
            name: v.name,
            url: v.url,
        }
    }
}

impl From<d::Recipient> for RecipientResponse {
    fn from(v: d::Recipient) -> Self {
        Self {
            name: v.name,
            postal_code: v.postal_code,
            city: v.city,
            address: v.address,
            phone_number: v.phone_number,
            email: v.email,
            shared_access_phone_number: v.shared_access_phone_number,
            bankid_authenticated: v.bankid_authenticated,
        }
    }
}

impl From<d::Event> for EventResponse {
    fn from(v: d::Event) -> Self {
        Self {
            description: v.description,
            date: v.date,
            city: v.city,
            country_code: v.country_code,
            r#type: v.r#type,
            cause: v.cause,
            whats_next_description: v.whats_next_description,
            display_status: v.display_status,
        }
    }
}

impl From<d::Feature> for FeatureResponse {
    fn from(v: d::Feature) -> Self {
        Self {
            r#type: v.r#type,
            url: v.url,
            title: v.title,
            description: v.description,
            date: v.date,
        }
    }
}

impl From<d::CustomsTax> for CustomsTaxResponse {
    fn from(v: d::CustomsTax) -> Self {
        Self {
            r#type: v.r#type,
            status: v.status,
            total_amount_in_ore: v.total_amount_in_ore,
            parcel_content: v.parcel_content.into_iter().map(Into::into).collect(),
            customs_price_list: v.customs_price_list.into_iter().map(Into::into).collect(),
            due_date: v.due_date,
        }
    }
}

impl From<d::ParcelContentItem> for ParcelContent {
    fn from(v: d::ParcelContentItem) -> Self {
        Self {
            description: v.description,
            currency_code: v.currency_code,
            amount_in_subunit: v.amount_in_subunit,
        }
    }
}

impl From<d::CustomsPriceItem> for CustomsPrice {
    fn from(v: d::CustomsPriceItem) -> Self {
        Self {
            description: v.description,
            amount_in_ore: v.amount_in_ore,
        }
    }
}

impl From<d::CustomsInformationRequirements> for CustomsInformationRequirements {
    fn from(v: d::CustomsInformationRequirements) -> Self {
        Self {
            documents_required: v.documents_required,
            information_provided: v.information_provided,
            identification_required: v.identification_required,
        }
    }
}

impl From<d::Rewards> for RewardsResponse {
    fn from(v: d::Rewards) -> Self {
        Self {
            rewards_earnings: v.rewards_earnings.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<d::RewardsEarning> for RewardsEarningResponse {
    fn from(v: d::RewardsEarning) -> Self {
        Self {
            valid_from: v.valid_from,
            valid_to: v.valid_to,
            r#type: v.r#type,
            text: v.text,
            coins: v.coins,
        }
    }
}
