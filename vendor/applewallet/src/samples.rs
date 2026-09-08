//! Ready-made, editable sample passes — one per Wallet style. The admin UI lists
//! [`sample_catalog`], lets you tweak the JSON, and signs the result.

use chrono::Utc;
use serde::Serialize;

use crate::builder::*;
use crate::model::*;

#[allow(clippy::too_many_arguments)]
fn boarding_pass(
    transit: PKTransitType,
    serial: &str,
    description: &str,
    org: &str,
    logo: &str,
    depart: chrono::DateTime<Utc>,
    from: &str,
    to: &str,
    passenger: &str,
    seat: &str,
    vehicle_label: &str,
    vehicle: &str,
    extra_label: &str,
    extra: &str,
    bg: &str,
    fg: &str,
    label: &str,
) -> PKPass {
    let exp = depart + chrono::Duration::hours(12);
    build_pass(PassConfig {
        description: description.to_string(),
        org_name: org.to_string(),
        serial: serial.to_string(),
        style: PKPassStyle::BoardingPass(PKPassBoardingStructure {
            transit_type: transit,
            header_fields: vec![date_field("date", "Date", depart)],
            primary_fields: vec![field("origin", "From", from), field("destination", "To", to)],
            secondary_fields: vec![
                field("passenger", "Passenger", passenger),
                field("seat", "Seat", seat),
            ],
            aux_fields: vec![
                field("vehicle", vehicle_label, vehicle),
                field("extra", extra_label, extra),
            ],
            back_fields: vec![
                field("depart", "Departure", &depart.format("%H:%M").to_string()),
                field("terms", "Terms", "Sample pass for Apple Wallet testing only."),
                field("ref", "Booking ref", serial),
            ],
        }),
        bg_colour: Some(bg.to_string()),
        fg_colour: Some(fg.to_string()),
        label_colour: Some(label.to_string()),
        logo_text: Some(logo.to_string()),
        exp_date: Some(exp),
        barcode_message: Some(serial.to_string()),
    })
}

pub fn sample_boarding_train() -> PKPass {
    boarding_pass(
        PKTransitType::Train, "SAMPLE-BOARDING-TRAIN", "Train — Dar es Salaam to Morogoro",
        "Sample Rail", "Sample Rail", tomorrow_at(8, 30), "Dar es Salaam", "Morogoro",
        "Sample Passenger", "12A", "Train", "Express 101", "Platform", "3",
        "rgb(0, 71, 142)", "rgb(255, 255, 255)", "rgb(255, 204, 0)",
    )
}

pub fn sample_boarding_air() -> PKPass {
    boarding_pass(
        PKTransitType::Air, "SAMPLE-BOARDING-AIR", "Flight — JRO to DAR",
        "Sample Airways", "Sample Air", tomorrow_at(14, 15), "Kilimanjaro", "Dar es Salaam",
        "Sample Passenger", "14C", "Flight", "SA 204", "Gate", "B7",
        "rgb(20, 40, 100)", "rgb(255, 255, 255)", "rgb(180, 210, 255)",
    )
}

pub fn sample_boarding_bus() -> PKPass {
    boarding_pass(
        PKTransitType::Bus, "SAMPLE-BOARDING-BUS", "Bus — Arusha to Moshi",
        "Sample Coach", "Sample Bus", tomorrow_at(9, 0), "Arusha", "Moshi",
        "Sample Passenger", "7", "Bus", "Line 42", "Terminal", "North",
        "rgb(0, 120, 60)", "rgb(255, 255, 255)", "rgb(200, 255, 200)",
    )
}

pub fn sample_boarding_boat() -> PKPass {
    boarding_pass(
        PKTransitType::Boat, "SAMPLE-BOARDING-BOAT", "Ferry — Zanzibar to Dar",
        "Sample Ferries", "Sample Ferry", tomorrow_at(11, 45), "Zanzibar", "Dar es Salaam",
        "Sample Passenger", "Deck A", "Vessel", "Island Star", "Deck", "Upper",
        "rgb(0, 90, 130)", "rgb(255, 255, 255)", "rgb(150, 220, 255)",
    )
}

pub fn sample_boarding_generic() -> PKPass {
    boarding_pass(
        PKTransitType::Generic, "SAMPLE-BOARDING-GENERIC", "Transit — City Centre to Airport",
        "Sample Transit", "Sample Transit", tomorrow_at(6, 0), "City Centre", "Airport",
        "Sample Passenger", "—", "Route", "A1 Express", "Zone", "3",
        "rgb(80, 80, 80)", "rgb(255, 255, 255)", "rgb(220, 220, 220)",
    )
}

pub fn sample_event_ticket() -> PKPass {
    let event_date = tomorrow_at(19, 30);
    let serial = "SAMPLE-EVENT-001";
    build_pass(PassConfig {
        description: "Concert — Sample Stadium".to_string(),
        org_name: "Sample Events".to_string(),
        serial: serial.to_string(),
        style: PKPassStyle::EventTicket(PKPassStructure {
            header_fields: vec![date_field("datetime", "When", event_date)],
            primary_fields: vec![field("event", "Event", "Summer Live Festival")],
            secondary_fields: vec![
                field("venue", "Venue", "Sample Stadium"),
                field("section", "Section", "VIP"),
            ],
            aux_fields: vec![field("row", "Row", "A"), field("seat", "Seat", "12")],
            back_fields: vec![
                field("doors", "Doors open", "18:30"),
                field("terms", "Terms", "Sample event ticket. Not valid for entry."),
            ],
        }),
        bg_colour: Some("rgb(90, 20, 120)".to_string()),
        fg_colour: Some("rgb(255, 255, 255)".to_string()),
        label_colour: Some("rgb(255, 180, 255)".to_string()),
        logo_text: Some("Sample Events".to_string()),
        exp_date: Some(event_date + chrono::Duration::hours(4)),
        barcode_message: Some(serial.to_string()),
    })
}

pub fn sample_coupon() -> PKPass {
    let serial = "SAMPLE-COUPON-001";
    let expires = Utc::now() + chrono::Duration::days(30);
    build_pass(PassConfig {
        description: "20% off — Sample Cafe".to_string(),
        org_name: "Sample Cafe".to_string(),
        serial: serial.to_string(),
        style: PKPassStyle::Coupon(PKPassStructure {
            header_fields: vec![date_field("expires", "Expires", expires)],
            primary_fields: vec![field("offer", "Offer", "20% Off Any Drink")],
            secondary_fields: vec![field("location", "Valid at", "All Sample Cafe stores")],
            aux_fields: vec![field("code", "Promo", "DRINK20")],
            back_fields: vec![field("terms", "Terms", "One use per customer. Sample coupon.")],
        }),
        bg_colour: Some("rgb(180, 40, 40)".to_string()),
        fg_colour: Some("rgb(255, 255, 255)".to_string()),
        label_colour: Some("rgb(255, 220, 180)".to_string()),
        logo_text: Some("Sample Cafe".to_string()),
        exp_date: Some(expires),
        barcode_message: Some(serial.to_string()),
    })
}

pub fn sample_store_card() -> PKPass {
    let serial = "SAMPLE-STORE-001";
    build_pass(PassConfig {
        description: "Loyalty card — Sample Market".to_string(),
        org_name: "Sample Market".to_string(),
        serial: serial.to_string(),
        style: PKPassStyle::StoreCard(PKPassStructure {
            header_fields: vec![field("member", "Member", "Gold")],
            primary_fields: vec![field("balance", "Points", "2,450")],
            secondary_fields: vec![field("name", "Name", "Sample Customer")],
            aux_fields: vec![field("tier", "Tier", "Gold Rewards")],
            back_fields: vec![
                field("id", "Member ID", "SM-88421"),
                field("terms", "Terms", "Sample loyalty card for testing only."),
            ],
        }),
        bg_colour: Some("rgb(30, 100, 50)".to_string()),
        fg_colour: Some("rgb(255, 255, 255)".to_string()),
        label_colour: Some("rgb(200, 255, 200)".to_string()),
        logo_text: Some("Sample Market".to_string()),
        exp_date: None,
        barcode_message: Some("SM-88421".to_string()),
    })
}

pub fn sample_generic() -> PKPass {
    let serial = "SAMPLE-GENERIC-001";
    build_pass(PassConfig {
        description: "Membership — Sample Club".to_string(),
        org_name: "Sample Club".to_string(),
        serial: serial.to_string(),
        style: PKPassStyle::Generic(PKPassStructure {
            header_fields: vec![field("status", "Status", "Active")],
            primary_fields: vec![field("member", "Member", "Sample Member")],
            secondary_fields: vec![field("id", "ID", "CLB-2026-001")],
            aux_fields: vec![field("since", "Member since", "2024")],
            back_fields: vec![field("terms", "Terms", "Sample membership pass for testing only.")],
        }),
        bg_colour: Some("rgb(50, 50, 60)".to_string()),
        fg_colour: Some("rgb(255, 255, 255)".to_string()),
        label_colour: Some("rgb(200, 200, 210)".to_string()),
        logo_text: Some("Sample Club".to_string()),
        exp_date: Some(Utc::now() + chrono::Duration::days(365)),
        barcode_message: Some(serial.to_string()),
    })
}

/// Catalog entry shown in the admin picker.
#[derive(Debug, Clone, Serialize)]
pub struct SamplePassInfo {
    pub id: String,
    pub name: String,
    pub pass_style: String,
    pub description: String,
    pub colour: String,
}

fn info(id: &str, name: &str, style: &str, description: &str, colour: &str) -> SamplePassInfo {
    SamplePassInfo {
        id: id.into(),
        name: name.into(),
        pass_style: style.into(),
        description: description.into(),
        colour: colour.into(),
    }
}

pub fn sample_catalog() -> Vec<SamplePassInfo> {
    vec![
        info("boarding-train", "Train", "boardingPass", "PKTransitTypeTrain — rail boarding pass", "rgb(0, 71, 142)"),
        info("boarding-air", "Flight", "boardingPass", "PKTransitTypeAir — flight boarding pass", "rgb(20, 40, 100)"),
        info("boarding-bus", "Bus", "boardingPass", "PKTransitTypeBus — coach boarding pass", "rgb(0, 120, 60)"),
        info("boarding-boat", "Ferry", "boardingPass", "PKTransitTypeBoat — ferry boarding pass", "rgb(0, 90, 130)"),
        info("boarding-generic", "Transit", "boardingPass", "PKTransitTypeGeneric — generic transit pass", "rgb(80, 80, 80)"),
        info("event", "Event Ticket", "eventTicket", "Concert / sports event admission", "rgb(90, 20, 120)"),
        info("coupon", "Coupon", "coupon", "Discount / promotional offer", "rgb(180, 40, 40)"),
        info("store-card", "Store Card", "storeCard", "Loyalty / rewards card", "rgb(30, 100, 50)"),
        info("generic", "Generic", "generic", "Membership / catch-all pass style", "rgb(50, 50, 60)"),
    ]
}

/// Resolve a catalog id to a fresh sample pass.
pub fn sample_pass_by_id(id: &str) -> Option<PKPass> {
    match id {
        "boarding-train" | "train" => Some(sample_boarding_train()),
        "boarding-air" => Some(sample_boarding_air()),
        "boarding-bus" => Some(sample_boarding_bus()),
        "boarding-boat" => Some(sample_boarding_boat()),
        "boarding-generic" => Some(sample_boarding_generic()),
        "event" => Some(sample_event_ticket()),
        "coupon" => Some(sample_coupon()),
        "store-card" => Some(sample_store_card()),
        "generic" => Some(sample_generic()),
        _ => None,
    }
}
