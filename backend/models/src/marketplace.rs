//! Service Marketplace (Fundi) models.
//! Organizations (providers), services, bookings, cart, custom requests,
//! bids, reviews, commissions, payouts, promotions, notifications.

use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::{
    marketplace_bids, marketplace_bookings, marketplace_cart, marketplace_categories,
    marketplace_commissions, marketplace_notifications, marketplace_org_members,
    marketplace_organizations, marketplace_payouts, marketplace_promotions, marketplace_requests,
    marketplace_reviews, marketplace_service_images, marketplace_services, marketplace_zones,
};

// ================================================ organizations (providers) ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_organizations)]
pub struct Organization {
    pub id: String,
    pub owner_user_id: String,
    pub name: String,
    pub slug: String,
    pub business_name: Option<String>,
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub timezone: Option<String>,
    pub base_currency: String,
    pub is_verified: i32,
    pub is_active: i32,
    pub rating_avg: f32,
    pub rating_count: i32,
    pub commission_rate: f32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_organizations)]
pub struct NewOrganization<'a> {
    pub id: &'a str,
    pub owner_user_id: &'a str,
    pub name: &'a str,
    pub slug: &'a str,
    pub business_name: Option<&'a str>,
    pub logo_url: Option<&'a str>,
    pub description: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub email: Option<&'a str>,
    pub website: Option<&'a str>,
    pub address: Option<&'a str>,
    pub city: Option<&'a str>,
    pub country: Option<&'a str>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub timezone: Option<&'a str>,
    pub base_currency: &'a str,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = marketplace_organizations)]
pub struct UpdateOrganization {
    pub name: Option<String>,
    pub business_name: Option<Option<String>>,
    pub logo_url: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub phone: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub website: Option<Option<String>>,
    pub address: Option<Option<String>>,
    pub city: Option<Option<String>>,
    pub country: Option<Option<String>>,
    pub latitude: Option<Option<f32>>,
    pub longitude: Option<Option<f32>>,
    pub timezone: Option<Option<String>>,
    pub base_currency: Option<String>,
    pub is_verified: Option<i32>,
    pub is_active: Option<i32>,
    pub commission_rate: Option<f32>,
}

// ====================================================== org members ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_org_members)]
pub struct OrgMember {
    pub id: String,
    pub org_id: String,
    pub user_id: String,
    pub role: String,
    pub is_active: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_org_members)]
pub struct NewOrgMember<'a> {
    pub id: &'a str,
    pub org_id: &'a str,
    pub user_id: &'a str,
    pub role: &'a str,
    pub is_active: i32,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = marketplace_org_members)]
pub struct UpdateOrgMember {
    pub role: Option<String>,
    pub is_active: Option<i32>,
}

// ================================================ service categories ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_categories)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub is_active: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_categories)]
pub struct NewCategory<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub slug: &'a str,
    pub description: Option<&'a str>,
    pub icon_url: Option<&'a str>,
    pub parent_id: Option<&'a str>,
    pub sort_order: i32,
    pub is_active: i32,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = marketplace_categories)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub icon_url: Option<Option<String>>,
    pub parent_id: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<i32>,
}

// ================================================ services ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_services)]
pub struct Service {
    pub id: String,
    pub org_id: String,
    pub category_id: Option<String>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub cover_image_url: Option<String>,
    pub price_type: String,
    pub base_price: f32,
    pub currency: String,
    pub duration_minutes: Option<i32>,
    pub is_active: i32,
    pub rating_avg: f32,
    pub rating_count: i32,
    pub booking_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_services)]
pub struct NewService<'a> {
    pub id: &'a str,
    pub org_id: &'a str,
    pub category_id: Option<&'a str>,
    pub name: &'a str,
    pub slug: &'a str,
    pub description: Option<&'a str>,
    pub short_description: Option<&'a str>,
    pub cover_image_url: Option<&'a str>,
    pub price_type: &'a str,
    pub base_price: f32,
    pub currency: &'a str,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = marketplace_services)]
pub struct UpdateService {
    pub name: Option<String>,
    pub category_id: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub short_description: Option<Option<String>>,
    pub cover_image_url: Option<Option<String>>,
    pub price_type: Option<String>,
    pub base_price: Option<f32>,
    pub currency: Option<String>,
    pub duration_minutes: Option<Option<i32>>,
    pub is_active: Option<i32>,
}

// ====================================================== service zones ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_zones)]
pub struct Zone {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub radius_km: Option<f32>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub is_active: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_zones)]
pub struct NewZone<'a> {
    pub id: &'a str,
    pub org_id: &'a str,
    pub name: &'a str,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub radius_km: Option<f32>,
    pub city: Option<&'a str>,
    pub country: Option<&'a str>,
}

// =============================================== service images ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_service_images)]
pub struct ServiceImage {
    pub id: String,
    pub service_id: String,
    pub url: String,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_service_images)]
pub struct NewServiceImage<'a> {
    pub id: &'a str,
    pub service_id: &'a str,
    pub url: &'a str,
    pub sort_order: i32,
}

// ======================================================== bookings ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_bookings)]
pub struct Booking {
    pub id: String,
    pub org_id: String,
    pub service_id: String,
    pub customer_user_id: String,
    pub serviceman_user_id: Option<String>,
    pub booking_type: String,
    pub status: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub address: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub notes: Option<String>,
    pub total_price: f32,
    pub currency: String,
    pub commission_amount: f32,
    pub provider_payout: f32,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub promo_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_bookings)]
pub struct NewBooking<'a> {
    pub id: &'a str,
    pub org_id: &'a str,
    pub service_id: &'a str,
    pub customer_user_id: &'a str,
    pub serviceman_user_id: Option<&'a str>,
    pub booking_type: &'a str,
    pub status: &'a str,
    pub scheduled_at: Option<NaiveDateTime>,
    pub address: Option<&'a str>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub notes: Option<&'a str>,
    pub total_price: f32,
    pub currency: &'a str,
    pub commission_amount: f32,
    pub provider_payout: f32,
    pub payment_method: Option<&'a str>,
    pub promo_id: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = marketplace_bookings)]
pub struct UpdateBooking {
    pub serviceman_user_id: Option<Option<String>>,
    pub status: Option<String>,
    pub scheduled_at: Option<Option<NaiveDateTime>>,
    pub started_at: Option<Option<NaiveDateTime>>,
    pub completed_at: Option<Option<NaiveDateTime>>,
    pub address: Option<Option<String>>,
    pub latitude: Option<Option<f32>>,
    pub longitude: Option<Option<f32>>,
    pub notes: Option<Option<String>>,
    pub total_price: Option<f32>,
    pub commission_amount: Option<f32>,
    pub provider_payout: Option<f32>,
    pub payment_status: Option<String>,
    pub payment_method: Option<Option<String>>,
}

// =================================================== custom requests ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_requests)]
pub struct Request {
    pub id: String,
    pub customer_user_id: String,
    pub category_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub budget_min: Option<f32>,
    pub budget_max: Option<f32>,
    pub currency: String,
    pub preferred_date: Option<NaiveDateTime>,
    pub status: String,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_requests)]
pub struct NewRequest<'a> {
    pub id: &'a str,
    pub customer_user_id: &'a str,
    pub category_id: Option<&'a str>,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub address: Option<&'a str>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub budget_min: Option<f32>,
    pub budget_max: Option<f32>,
    pub currency: &'a str,
    pub preferred_date: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
}

// ====================================================== provider bids ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_bids)]
pub struct Bid {
    pub id: String,
    pub request_id: String,
    pub org_id: String,
    pub price: f32,
    pub currency: String,
    pub message: Option<String>,
    pub estimated_days: Option<i32>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_bids)]
pub struct NewBid<'a> {
    pub id: &'a str,
    pub request_id: &'a str,
    pub org_id: &'a str,
    pub price: f32,
    pub currency: &'a str,
    pub message: Option<&'a str>,
    pub estimated_days: Option<i32>,
}

// ============================================================ cart ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_cart)]
pub struct CartItem {
    pub id: String,
    pub user_id: String,
    pub service_id: String,
    pub quantity: i32,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_cart)]
pub struct NewCartItem<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub service_id: &'a str,
    pub quantity: i32,
    pub notes: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = marketplace_cart)]
pub struct UpdateCartItem {
    pub quantity: Option<i32>,
    pub notes: Option<Option<String>>,
}

// =============================================== reviews & ratings ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_reviews)]
pub struct Review {
    pub id: String,
    pub booking_id: String,
    pub org_id: String,
    pub service_id: Option<String>,
    pub customer_user_id: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_reviews)]
pub struct NewReview<'a> {
    pub id: &'a str,
    pub booking_id: &'a str,
    pub org_id: &'a str,
    pub service_id: Option<&'a str>,
    pub customer_user_id: &'a str,
    pub rating: i32,
    pub comment: Option<&'a str>,
}

// ============================================== commissions ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_commissions)]
pub struct Commission {
    pub id: String,
    pub booking_id: String,
    pub org_id: String,
    pub amount: f32,
    pub rate: f32,
    pub currency: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_commissions)]
pub struct NewCommission<'a> {
    pub id: &'a str,
    pub booking_id: &'a str,
    pub org_id: &'a str,
    pub amount: f32,
    pub rate: f32,
    pub currency: &'a str,
}

// ============================================ provider payouts ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_payouts)]
pub struct Payout {
    pub id: String,
    pub org_id: String,
    pub amount: f32,
    pub currency: String,
    pub method: String,
    pub reference: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_payouts)]
pub struct NewPayout<'a> {
    pub id: &'a str,
    pub org_id: &'a str,
    pub amount: f32,
    pub currency: &'a str,
    pub method: &'a str,
    pub reference: Option<&'a str>,
}

// ============================================== promotions ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_promotions)]
pub struct Promotion {
    pub id: String,
    pub org_id: Option<String>,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: f32,
    pub min_booking_value: Option<f32>,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub starts_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub is_active: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_promotions)]
pub struct NewPromotion<'a> {
    pub id: &'a str,
    pub org_id: Option<&'a str>,
    pub code: &'a str,
    pub description: Option<&'a str>,
    pub discount_type: &'a str,
    pub discount_value: f32,
    pub min_booking_value: Option<f32>,
    pub max_uses: Option<i32>,
    pub starts_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = marketplace_promotions)]
pub struct UpdatePromotion {
    pub description: Option<Option<String>>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f32>,
    pub min_booking_value: Option<Option<f32>>,
    pub max_uses: Option<Option<i32>>,
    pub is_active: Option<i32>,
}

// ============================================ notifications ==

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = marketplace_notifications)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub data: Option<String>,
    pub is_read: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = marketplace_notifications)]
pub struct NewNotification<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub kind: &'a str,
    pub title: &'a str,
    pub body: Option<&'a str>,
    pub data: Option<&'a str>,
}
