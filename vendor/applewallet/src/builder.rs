//! Ergonomic helpers for assembling a [`PKPass`] in code.
//!
//! `type_id`, `team_id`, `web_service_url` and `authentication_token` are left
//! empty here — they are stamped in at sign time from [`crate::config::WalletConfig`]
//! so the same pass body works against any team/server.

use chrono::{DateTime, Timelike, Utc};

use crate::model::*;

/// A plain text field.
pub fn field(key: &str, label: &str, value: &str) -> PKPassField {
    PKPassField {
        key: key.to_string(),
        label: Some(label.to_string()),
        value: value.to_string(),
        ..Default::default()
    }
}

/// A date/time field (medium date, short time, fixed time zone).
pub fn date_field(key: &str, label: &str, dt: DateTime<Utc>) -> PKPassField {
    PKPassField {
        key: key.to_string(),
        label: Some(label.to_string()),
        value: dt.to_rfc3339(),
        date_style: Some(PKDateStyle::Medium),
        time_style: Some(PKDateStyle::Short),
        ignores_time_zone: Some(true),
        ..Default::default()
    }
}

/// Inputs for [`build_pass`].
#[derive(Debug, Clone)]
pub struct PassConfig {
    pub description: String,
    pub org_name: String,
    pub serial: String,
    pub style: PKPassStyle,
    pub bg_colour: Option<String>,
    pub fg_colour: Option<String>,
    pub label_colour: Option<String>,
    pub logo_text: Option<String>,
    pub exp_date: Option<DateTime<Utc>>,
    pub barcode_message: Option<String>,
}

/// Assemble a [`PKPass`]. A QR barcode is added when `barcode_message` is set.
pub fn build_pass(config: PassConfig) -> PKPass {
    let barcode = config.barcode_message.as_ref().map(|msg| PKPassBarcode {
        format: PKBarcodeFormat::QR,
        message: msg.clone(),
        message_encoding: "iso-8859-1".to_string(),
        alt_text: Some(msg.clone()),
    });

    PKPass {
        format_version: 1,
        description: config.description,
        org_name: config.org_name,
        type_id: String::new(),
        serial: config.serial,
        team_id: String::new(),
        web_service_url: None,
        authentication_token: None,
        voided: false,
        pass_style: config.style,
        bg_colour: config.bg_colour,
        fg_colour: config.fg_colour,
        label_colour: config.label_colour,
        logo_text: config.logo_text,
        exp_date: config.exp_date,
        barcode: barcode.clone(),
        barcodes: barcode.into_iter().collect(),
    }
}

/// Tomorrow at `hour:minute` UTC — handy for sample dates.
pub fn tomorrow_at(hour: u32, minute: u32) -> DateTime<Utc> {
    let base = Utc::now() + chrono::Duration::days(1);
    base.with_hour(hour)
        .and_then(|t| t.with_minute(minute))
        .and_then(|t| t.with_second(0))
        .unwrap_or(base)
}
