use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::Pg;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema::{subscription_plans, organization_subscriptions, subscription_invoices};

// ---------------------------------------------------------- Plans --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = subscription_plans)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price_monthly: f32,
    pub price_yearly: f32,
    pub currency: String,
    pub max_users: i32,
    pub max_products: i32,
    pub max_orders: i32,
    pub features: String,
    pub is_active: i32,
    pub is_public: i32,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = subscription_plans)]
pub struct NewPlan {
    pub id: Option<String>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price_monthly: f32,
    pub price_yearly: f32,
    pub currency: String,
    pub max_users: i32,
    pub max_products: i32,
    pub max_orders: i32,
    pub features: String,
    pub is_active: i32,
    pub is_public: i32,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = subscription_plans)]
pub struct UpdatePlan {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub price_monthly: Option<f32>,
    pub price_yearly: Option<f32>,
    pub currency: Option<String>,
    pub max_users: Option<i32>,
    pub max_products: Option<i32>,
    pub max_orders: Option<i32>,
    pub features: Option<String>,
    pub is_active: Option<i32>,
    pub is_public: Option<i32>,
    pub sort_order: Option<i32>,
}

// ---------------------------------------------------------- Organization Subscriptions --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = organization_subscriptions)]
pub struct Subscription {
    pub id: String,
    pub org_id: String,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: NaiveDateTime,
    pub current_period_end: NaiveDateTime,
    pub cancel_at_period_end: i32,
    pub payment_method: Option<String>,
    pub trial_ends_at: Option<NaiveDateTime>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = organization_subscriptions)]
pub struct NewSubscription {
    pub org_id: String,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: NaiveDateTime,
    pub current_period_end: NaiveDateTime,
    pub payment_method: Option<String>,
}

// ---------------------------------------------------------- Invoices --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = subscription_invoices)]
#[diesel(check_for_backend(Pg))]
pub struct Invoice {
    pub id: String,
    pub subscription_id: String,
    pub org_id: String,
    pub amount: f32,
    pub currency: String,
    pub status: String,
    pub due_date: NaiveDateTime,
    pub paid_at: Option<NaiveDateTime>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = subscription_invoices)]
pub struct NewInvoice {
    pub subscription_id: String,
    pub org_id: String,
    pub amount: f32,
    pub currency: String,
    pub status: String,
    pub due_date: NaiveDateTime,
    pub description: Option<String>,
}
