use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parcel {
    pub parcel_number: String,
    pub consignment_number: String,
    pub status: ParcelStatus,
    pub user_chosen_name: Option<String>,
    pub direction: Direction,
    pub transport: Option<Transport>,
    pub dimensions: Option<Dimensions>,
    pub weight_in_kg: Option<f64>,
    pub delivery: Option<Delivery>,
    pub sender: Option<Sender>,
    pub recipient: Option<Recipient>,
    pub product_name: ProductName,
    pub product_group: ProductGroup,
    pub events: Vec<Event>,
    #[serde(default)]
    pub features: Vec<Feature>,
    pub expires_at: DateTime<Utc>,
    pub parcel_numbers_in_consignment: Vec<String>,
    pub customs_tax: Option<CustomsTax>,
    pub customs_information_requirements: Option<CustomsInformationRequirements>,
    pub rewards: Option<Rewards>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParcelStatus {
    Notified,
    Underway,
    Collectable,
    ReturnUnderway,
    ReturnCollectable,
    Archived,
    ArchivedByUser,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Sent,
    Receive,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductGroup {
    Return,
    Mailbox,
    PickupPoint,
    HomeDelivery,
    ParcelLocker,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductName {
    NorwayParcel,
    NorwayParcelExpress,
    NorwayParcelSmall,
    ParcelPrivateToMailbox,
    ParcelPrivateToParcelLocker,
    ParcelPrivateToPickupPoint,
    BusinessParcelStandard,
    BusinessParcelExpressOvernight,
    HomeDeliveryParcelReturn,
    PickupParcel,
    PickupParcelLocker,
    RegisteredLetter,
    BringPack,
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transport {
    pub r#type: String,
    pub electric: bool,
    pub fuel_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimensions {
    pub length_in_cm: i32,
    pub width_in_cm: i32,
    pub height_in_cm: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    pub r#type: String,
    pub pick_up_point_id: Option<String>,
    pub deadline_date: Option<DateTime<Utc>>,
    pub extend_deadline_to: Option<DateTime<Utc>>,
    pub pick_up_code: Option<String>,
    pub pick_up_qr_code: Option<String>,
    pub shelf_number: Option<String>,
    pub gate_code: Option<String>,
    pub pin_code: Option<String>,
    pub qr_code: Option<String>,
    pub permission: String,
    pub bankid_authenticated: String,
    pub options: Vec<String>,
    pub progress_percentage_based_on_events: i32,
    pub delivery_time: Option<DeliveryTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryTime {
    pub date: DateTime<Utc>,
    pub delivery_window: Option<DeliveryWindow>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sender {
    pub name: String,
    pub icon_url: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub customer_number: Option<String>,
    pub branding: Option<Branding>,
    pub customer_service: Option<CustomerService>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Branding {
    pub banner_image: Option<Image>,
    pub content_image: Option<Image>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub store_link: Option<Link>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub url: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub url: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerService {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipient {
    pub name: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub shared_access_phone_number: Option<String>,
    pub bankid_authenticated: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub description: String,
    pub date: DateTime<Utc>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub r#type: String,
    pub cause: Option<String>,
    pub whats_next_description: Option<String>,
    pub display_status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub r#type: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomsTax {
    pub r#type: String,
    pub status: String,
    pub total_amount_in_ore: i32,
    pub parcel_content: Vec<ParcelContentItem>,
    pub customs_price_list: Vec<CustomsPriceItem>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParcelContentItem {
    pub description: String,
    pub currency_code: String,
    pub amount_in_subunit: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomsPriceItem {
    pub description: String,
    pub amount_in_ore: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomsInformationRequirements {
    pub documents_required: bool,
    pub information_provided: bool,
    pub identification_required: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rewards {
    pub rewards_earnings: Vec<RewardsEarning>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardsEarning {
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub r#type: String,
    pub text: String,
    pub coins: i32,
}
