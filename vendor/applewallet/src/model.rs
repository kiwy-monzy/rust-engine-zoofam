//! Strongly-typed Apple Wallet pass model (`pass.json`).
//!
//! Every field is public and round-trips through serde, so a pass can be built
//! programmatically (see [`crate::samples`]) *or* edited as JSON in the admin UI
//! and re-signed. Optional fields are skipped when empty to keep `pass.json`
//! clean (Apple is strict about unexpected keys).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKPass {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    pub description: String,
    #[serde(rename = "organizationName")]
    pub org_name: String,
    #[serde(rename = "passTypeIdentifier")]
    pub type_id: String,
    #[serde(rename = "serialNumber")]
    pub serial: String,
    #[serde(rename = "teamIdentifier")]
    pub team_id: String,

    // Update web service — present only on updatable passes.
    #[serde(rename = "webServiceURL", skip_serializing_if = "Option::is_none", default)]
    pub web_service_url: Option<String>,
    #[serde(
        rename = "authenticationToken",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub authentication_token: Option<String>,

    #[serde(
        rename = "expirationDate",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub exp_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub voided: bool,

    #[serde(flatten)]
    pub pass_style: PKPassStyle,

    #[serde(rename = "backgroundColor", skip_serializing_if = "Option::is_none", default)]
    pub bg_colour: Option<String>,
    #[serde(rename = "foregroundColor", skip_serializing_if = "Option::is_none", default)]
    pub fg_colour: Option<String>,
    #[serde(rename = "labelColor", skip_serializing_if = "Option::is_none", default)]
    pub label_colour: Option<String>,
    #[serde(rename = "logoText", skip_serializing_if = "Option::is_none", default)]
    pub logo_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub barcode: Option<PKPassBarcode>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub barcodes: Vec<PKPassBarcode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PKPassStyle {
    #[serde(rename = "boardingPass")]
    BoardingPass(PKPassBoardingStructure),
    #[serde(rename = "coupon")]
    Coupon(PKPassStructure),
    #[serde(rename = "eventTicket")]
    EventTicket(PKPassStructure),
    #[serde(rename = "generic")]
    Generic(PKPassStructure),
    #[serde(rename = "storeCard")]
    StoreCard(PKPassStructure),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PKPassBoardingStructure {
    #[serde(rename = "transitType")]
    pub transit_type: PKTransitType,
    #[serde(rename = "auxiliaryFields", skip_serializing_if = "Vec::is_empty", default)]
    pub aux_fields: Vec<PKPassField>,
    #[serde(rename = "backFields", skip_serializing_if = "Vec::is_empty", default)]
    pub back_fields: Vec<PKPassField>,
    #[serde(rename = "headerFields", skip_serializing_if = "Vec::is_empty", default)]
    pub header_fields: Vec<PKPassField>,
    #[serde(rename = "primaryFields", skip_serializing_if = "Vec::is_empty", default)]
    pub primary_fields: Vec<PKPassField>,
    #[serde(rename = "secondaryFields", skip_serializing_if = "Vec::is_empty", default)]
    pub secondary_fields: Vec<PKPassField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PKPassStructure {
    #[serde(rename = "auxiliaryFields", skip_serializing_if = "Vec::is_empty", default)]
    pub aux_fields: Vec<PKPassField>,
    #[serde(rename = "backFields", skip_serializing_if = "Vec::is_empty", default)]
    pub back_fields: Vec<PKPassField>,
    #[serde(rename = "headerFields", skip_serializing_if = "Vec::is_empty", default)]
    pub header_fields: Vec<PKPassField>,
    #[serde(rename = "primaryFields", skip_serializing_if = "Vec::is_empty", default)]
    pub primary_fields: Vec<PKPassField>,
    #[serde(rename = "secondaryFields", skip_serializing_if = "Vec::is_empty", default)]
    pub secondary_fields: Vec<PKPassField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PKTransitType {
    #[serde(rename = "PKTransitTypeAir")]
    Air,
    #[serde(rename = "PKTransitTypeTrain")]
    Train,
    #[serde(rename = "PKTransitTypeBus")]
    Bus,
    #[serde(rename = "PKTransitTypeBoat")]
    Boat,
    #[default]
    #[serde(rename = "PKTransitTypeGeneric")]
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PKPassField {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub label: Option<String>,
    pub value: String,
    #[serde(rename = "dateStyle", skip_serializing_if = "Option::is_none", default)]
    pub date_style: Option<PKDateStyle>,
    #[serde(rename = "timeStyle", skip_serializing_if = "Option::is_none", default)]
    pub time_style: Option<PKDateStyle>,
    #[serde(rename = "textAlignment", skip_serializing_if = "Option::is_none", default)]
    pub text_alignment: Option<PKTextAlignment>,
    #[serde(rename = "ignoresTimeZone", skip_serializing_if = "Option::is_none", default)]
    pub ignores_time_zone: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PKBarcodeFormat {
    #[serde(rename = "PKBarcodeFormatQR")]
    QR,
    #[serde(rename = "PKBarcodeFormatPDF417")]
    PDF417,
    #[serde(rename = "PKBarcodeFormatAztec")]
    Aztec,
    #[serde(rename = "PKBarcodeFormatCode128")]
    Code128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PKTextAlignment {
    #[serde(rename = "PKTextAlignmentLeft")]
    Left,
    #[serde(rename = "PKTextAlignmentCenter")]
    Center,
    #[serde(rename = "PKTextAlignmentRight")]
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PKDateStyle {
    #[serde(rename = "PKDateStyleNone")]
    None,
    #[serde(rename = "PKDateStyleShort")]
    Short,
    #[serde(rename = "PKDateStyleMedium")]
    Medium,
    #[serde(rename = "PKDateStyleLong")]
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKPassBarcode {
    pub format: PKBarcodeFormat,
    pub message: String,
    #[serde(rename = "messageEncoding")]
    pub message_encoding: String,
    #[serde(rename = "altText", skip_serializing_if = "Option::is_none", default)]
    pub alt_text: Option<String>,
}
