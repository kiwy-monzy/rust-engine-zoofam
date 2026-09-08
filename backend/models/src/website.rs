//! Website templates (template 0 = Knowlia download-hub) + hero/slideshow/downloads/products/cart/bookings/links/theme.

use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::{
    website_bookings, website_cart_items, website_links, website_sections, website_templates,
};

// ------------------------------------------------ templates --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = website_templates)]
pub struct WebsiteTemplate {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub version: String,
    pub is_active: i32,
    pub theme_json: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = website_templates)]
pub struct NewWebsiteTemplate {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub version: String,
    pub is_active: i32,
    pub theme_json: Option<String>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = website_templates)]
pub struct UpdateWebsiteTemplate {
    pub name: Option<String>,
    pub version: Option<String>,
    pub is_active: Option<i32>,
    pub theme_json: Option<String>,
}

// ------------------------------------------------ sections --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = website_sections)]
pub struct WebsiteSection {
    pub id: String,
    pub template_id: String,
    pub kind: String,
    pub position: i32,
    pub config_json: String,
    pub is_visible: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = website_sections)]
pub struct NewWebsiteSection {
    pub id: String,
    pub template_id: String,
    pub kind: String,
    pub position: i32,
    pub config_json: String,
    pub is_visible: i32,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = website_sections)]
pub struct UpdateWebsiteSection {
    pub config_json: Option<String>,
    pub is_visible: Option<i32>,
    pub position: Option<i32>,
}

// ------------------------------------------------ links --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = website_links)]
pub struct WebsiteLink {
    pub id: String,
    pub template_id: String,
    pub label: String,
    pub href: String,
    pub icon: Option<String>,
    pub position: i32,
    pub is_visible: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = website_links)]
pub struct NewWebsiteLink {
    pub id: String,
    pub template_id: String,
    pub label: String,
    pub href: String,
    pub icon: Option<String>,
    pub position: i32,
    pub is_visible: i32,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = website_links)]
pub struct UpdateWebsiteLink {
    pub label: Option<String>,
    pub href: Option<String>,
    pub icon: Option<Option<String>>,
    pub position: Option<i32>,
    pub is_visible: Option<i32>,
}

// ------------------------------------------------ cart items (per-user, per-template) --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = website_cart_items)]
pub struct WebsiteCartItem {
    pub id: String,
    pub template_id: String,
    pub user_id: String,
    pub product_id: String,
    pub quantity: f32,
    pub unit_price: f32,
    pub added_at: NaiveDateTime,
}

#[derive(Queryable, Serialize)]
pub struct WebsiteCartRow {
    #[serde(flatten)]
    pub item: WebsiteCartItem,
    pub product_name: Option<String>,
    pub sku: Option<String>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = website_cart_items)]
pub struct NewWebsiteCartItem {
    pub id: String,
    pub template_id: String,
    pub user_id: String,
    pub product_id: String,
    pub quantity: f32,
    pub unit_price: f32,
}

// ------------------------------------------------ bookings --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = website_bookings)]
pub struct WebsiteBooking {
    pub id: String,
    pub template_id: String,
    pub user_id: Option<String>,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub service: Option<String>,
    pub product_id: Option<String>,
    pub date: NaiveDateTime,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = website_bookings)]
pub struct NewWebsiteBooking {
    pub id: String,
    pub template_id: String,
    pub user_id: Option<String>,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub service: Option<String>,
    pub product_id: Option<String>,
    pub date: NaiveDateTime,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = website_bookings)]
pub struct UpdateWebsiteBooking {
    pub customer_name: Option<String>,
    pub customer_email: Option<Option<String>>,
    pub service: Option<Option<String>>,
    pub product_id: Option<Option<String>>,
    pub date: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub notes: Option<Option<String>>,
}
