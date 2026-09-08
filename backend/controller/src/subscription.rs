//! Subscription controller - manages plans, organization subscriptions, and invoices

use chrono::{Duration, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use db::DbPool;
use models::subscription::{NewPlan, NewSubscription, Plan, Subscription, UpdatePlan};

// Import schema tables
use models::schema::subscription_plans;
use models::schema::organization_subscriptions;
use models::schema::subscription_invoices;

// ---------------------------------------------------------- Plans --

pub fn list_plans(pool: &DbPool) -> Result<Vec<Plan>, diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    subscription_plans::table
        .filter(subscription_plans::is_active.eq(1i32))
        .filter(subscription_plans::is_public.eq(1i32))
        .order(subscription_plans::sort_order.asc())
        .load::<Plan>(&mut conn)
}

pub fn create_plan(pool: &DbPool, mut new: NewPlan) -> Result<Plan, diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    let id_val = Uuid::new_v4().to_string();
    new.id = Some(id_val.clone());
    diesel::insert_into(subscription_plans::table)
        .values(&new)
        .execute(&mut conn)?;
    subscription_plans::table.find(id_val).first(&mut conn)
}

pub fn update_plan(pool: &DbPool, plan_id: &str, upd: UpdatePlan) -> Result<Plan, diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    diesel::update(subscription_plans::table.find(plan_id))
        .set(&upd)
        .execute(&mut conn)?;
    subscription_plans::table.find(plan_id).first(&mut conn)
}

pub fn delete_plan(pool: &DbPool, plan_id: &str) -> Result<(), diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    diesel::delete(subscription_plans::table.find(plan_id)).execute(&mut conn)?;
    Ok(())
}

// ---------------------------------------------------------- Organization Subscriptions --

pub fn get_organization_subscription(pool: &DbPool, org_id_val: &str) -> Result<Subscription, diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    organization_subscriptions::table
        .filter(organization_subscriptions::org_id.eq(org_id_val))
        .filter(organization_subscriptions::status.ne("cancelled"))
        .order(organization_subscriptions::created_at.desc())
        .first(&mut conn)
}

pub fn subscribe(pool: &DbPool, org_id_val: &str, plan_id_val: &str, period: &str) -> Result<Subscription, diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    let now = Utc::now().naive_utc();
    let duration_days = if period == "yearly" { 365 } else { 30 };
    let end_date = now + Duration::days(duration_days);

    let new_sub = NewSubscription {
        org_id: org_id_val.to_string(),
        plan_id: plan_id_val.to_string(),
        status: "active".to_string(),
        current_period_start: now,
        current_period_end: end_date,
        payment_method: None,
    };

    diesel::insert_into(organization_subscriptions::table)
        .values(&new_sub)
        .execute(&mut conn)?;
    organization_subscriptions::table
        .filter(organization_subscriptions::org_id.eq(org_id_val))
        .order(organization_subscriptions::created_at.desc())
        .first(&mut conn)
}

pub fn cancel(pool: &DbPool, org_id_val: &str) -> Result<(), diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    diesel::update(organization_subscriptions::table.filter(organization_subscriptions::org_id.eq(org_id_val)))
        .set((
            organization_subscriptions::status.eq("cancelled"),
            organization_subscriptions::cancelled_at.eq(Some(Utc::now().naive_utc())),
            organization_subscriptions::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;
    Ok(())
}

// ---------------------------------------------------------- Invoices --

pub fn list_invoices(pool: &DbPool, org_id_val: &str) -> Result<Vec<models::subscription::Invoice>, diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(e.to_string()),
    ))?;
    subscription_invoices::table
        .filter(subscription_invoices::org_id.eq(org_id_val))
        .order(subscription_invoices::created_at.desc())
        .load::<models::subscription::Invoice>(&mut conn)
}
