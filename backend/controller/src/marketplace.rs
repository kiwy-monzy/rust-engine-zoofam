//! Service Marketplace (Fundi) controller.
//! Organizations-as-providers, services, bookings, cart, custom requests,
//! bids, reviews, commissions, payouts, promotions, notifications.

use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use db::{conn, DbPool};
use models::marketplace::*;
use models::schema::{
    marketplace_bids, marketplace_bookings, marketplace_cart, marketplace_categories,
    marketplace_commissions, marketplace_notifications, marketplace_org_members,
    marketplace_organizations, marketplace_payouts, marketplace_promotions, marketplace_requests,
    marketplace_reviews, marketplace_services, marketplace_zones,
};

use crate::{Error, Result};

fn nid() -> String {
    Uuid::new_v4().to_string()
}
fn now() -> chrono::NaiveDateTime {
    Utc::now().naive_utc()
}

// ============================================================ organizations ===

pub fn list_organizations(pool: &DbPool) -> Result<Vec<Organization>> {
    let mut c = conn(pool)?;
    Ok(marketplace_organizations::table
        .order(marketplace_organizations::name.asc())
        .load::<Organization>(&mut c)?)
}

pub fn get_organization(pool: &DbPool, id: &str) -> Result<Organization> {
    let mut c = conn(pool)?;
    marketplace_organizations::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("organization"))
}

pub fn create_organization(
    pool: &DbPool,
    owner_user_id: &str,
    name: &str,
    slug: &str,
    business_name: Option<&str>,
    logo_url: Option<&str>,
    description: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    website: Option<&str>,
    address: Option<&str>,
    city: Option<&str>,
    country: Option<&str>,
    latitude: Option<f32>,
    longitude: Option<f32>,
    timezone: Option<&str>,
    base_currency: &str,
) -> Result<Organization> {
    let id = nid();
    let new = NewOrganization {
        id: &id,
        owner_user_id,
        name,
        slug,
        business_name,
        logo_url,
        description,
        phone,
        email,
        website,
        address,
        city,
        country,
        latitude,
        longitude,
        timezone,
        base_currency,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_organizations::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_organizations::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_organization(
    pool: &DbPool,
    id: &str,
    patch: &UpdateOrganization,
) -> Result<Organization> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_organizations::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    get_organization(pool, id)
}

pub fn delete_organization(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_organizations::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("organization"));
    }
    Ok(())
}

// ============================================================ org members ===

pub fn list_org_members(pool: &DbPool, org_id: &str) -> Result<Vec<OrgMember>> {
    let mut c = conn(pool)?;
    Ok(marketplace_org_members::table
        .filter(marketplace_org_members::org_id.eq(org_id))
        .order(marketplace_org_members::created_at.asc())
        .load::<OrgMember>(&mut c)?)
}

pub fn add_org_member(pool: &DbPool, org_id: &str, user_id: &str, role: &str) -> Result<OrgMember> {
    let id = nid();
    let new = NewOrgMember {
        id: &id,
        org_id,
        user_id,
        role,
        is_active: 1i32,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_org_members::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_org_members::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_org_member(pool: &DbPool, id: &str, patch: &UpdateOrgMember) -> Result<OrgMember> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_org_members::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    marketplace_org_members::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("org member"))
}

pub fn remove_org_member(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_org_members::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("org member"));
    }
    Ok(())
}

// ============================================================ categories ===

pub fn list_categories(pool: &DbPool) -> Result<Vec<Category>> {
    let mut c = conn(pool)?;
    Ok(marketplace_categories::table
        .order(marketplace_categories::sort_order.asc())
        .load::<Category>(&mut c)?)
}

pub fn get_category(pool: &DbPool, id: &str) -> Result<Category> {
    let mut c = conn(pool)?;
    marketplace_categories::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("category"))
}

pub fn create_category(
    pool: &DbPool,
    name: &str,
    slug: &str,
    description: Option<&str>,
    icon_url: Option<&str>,
    parent_id: Option<&str>,
    sort_order: i32,
) -> Result<Category> {
    let id = nid();
    let new = NewCategory {
        id: &id,
        name,
        slug,
        description,
        icon_url,
        parent_id,
        sort_order,
        is_active: 1i32,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_categories::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_categories::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_category(pool: &DbPool, id: &str, patch: &UpdateCategory) -> Result<Category> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_categories::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    get_category(pool, id)
}

pub fn delete_category(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_categories::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("category"));
    }
    Ok(())
}

// ============================================================ services ===

pub fn list_services(pool: &DbPool) -> Result<Vec<Service>> {
    let mut c = conn(pool)?;
    Ok(marketplace_services::table
        .order(marketplace_services::created_at.desc())
        .load::<Service>(&mut c)?)
}

pub fn list_services_by_org(pool: &DbPool, org_id: &str) -> Result<Vec<Service>> {
    let mut c = conn(pool)?;
    Ok(marketplace_services::table
        .filter(marketplace_services::org_id.eq(org_id))
        .order(marketplace_services::created_at.desc())
        .load::<Service>(&mut c)?)
}

pub fn get_service(pool: &DbPool, id: &str) -> Result<Service> {
    let mut c = conn(pool)?;
    marketplace_services::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("service"))
}

pub fn create_service(
    pool: &DbPool,
    org_id: &str,
    category_id: Option<&str>,
    name: &str,
    slug: &str,
    description: Option<&str>,
    short_description: Option<&str>,
    cover_image_url: Option<&str>,
    price_type: &str,
    base_price: f32,
    currency: &str,
    duration_minutes: Option<i32>,
) -> Result<Service> {
    let id = nid();
    let new = NewService {
        id: &id,
        org_id,
        category_id,
        name,
        slug,
        description,
        short_description,
        cover_image_url,
        price_type,
        base_price,
        currency,
        duration_minutes,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_services::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_services::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_service(pool: &DbPool, id: &str, patch: &UpdateService) -> Result<Service> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_services::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    get_service(pool, id)
}

pub fn delete_service(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_services::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("service"));
    }
    Ok(())
}

// ============================================================ zones ===

pub fn list_zones(pool: &DbPool, org_id: &str) -> Result<Vec<Zone>> {
    let mut c = conn(pool)?;
    Ok(marketplace_zones::table
        .filter(marketplace_zones::org_id.eq(org_id))
        .order(marketplace_zones::name.asc())
        .load::<Zone>(&mut c)?)
}

pub fn create_zone(
    pool: &DbPool,
    org_id: &str,
    name: &str,
    latitude: Option<f64>,
    longitude: Option<f64>,
    radius_km: Option<f64>,
    city: Option<&str>,
    country: Option<&str>,
) -> Result<Zone> {
    let id = nid();
    let new = NewZone {
        id: &id,
        org_id,
        name,
        latitude: latitude.map(|v| v as f32),
        longitude: longitude.map(|v| v as f32),
        radius_km: radius_km.map(|v| v as f32),
        city,
        country,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_zones::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_zones::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_zone(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_zones::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("zone"));
    }
    Ok(())
}

// ============================================================ bookings ===

pub fn list_bookings(pool: &DbPool) -> Result<Vec<Booking>> {
    let mut c = conn(pool)?;
    Ok(marketplace_bookings::table
        .order(marketplace_bookings::created_at.desc())
        .load::<Booking>(&mut c)?)
}

pub fn list_bookings_by_org(pool: &DbPool, org_id: &str) -> Result<Vec<Booking>> {
    let mut c = conn(pool)?;
    Ok(marketplace_bookings::table
        .filter(marketplace_bookings::org_id.eq(org_id))
        .order(marketplace_bookings::created_at.desc())
        .load::<Booking>(&mut c)?)
}

pub fn list_bookings_by_customer(pool: &DbPool, user_id: &str) -> Result<Vec<Booking>> {
    let mut c = conn(pool)?;
    Ok(marketplace_bookings::table
        .filter(marketplace_bookings::customer_user_id.eq(user_id))
        .order(marketplace_bookings::created_at.desc())
        .load::<Booking>(&mut c)?)
}

pub fn get_booking(pool: &DbPool, id: &str) -> Result<Booking> {
    let mut c = conn(pool)?;
    marketplace_bookings::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("booking"))
}

pub fn create_booking(
    pool: &DbPool,
    org_id: &str,
    service_id: &str,
    customer_user_id: &str,
    booking_type: &str,
    scheduled_at: Option<chrono::NaiveDateTime>,
    address: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    notes: Option<&str>,
    total_price: f64,
    currency: &str,
    commission_amount: f64,
    provider_payout: f64,
    payment_method: Option<&str>,
    promo_id: Option<&str>,
) -> Result<Booking> {
    let id = nid();
    let new = NewBooking {
        id: &id,
        org_id,
        service_id,
        customer_user_id,
        serviceman_user_id: None,
        booking_type,
        status: "pending",
        scheduled_at,
        address,
        latitude: latitude.map(|v| v as f32),
        longitude: longitude.map(|v| v as f32),
        notes,
        total_price: total_price as f32,
        currency,
        commission_amount: commission_amount as f32,
        provider_payout: provider_payout as f32,
        payment_method,
        promo_id,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_bookings::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_bookings::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_booking(pool: &DbPool, id: &str, patch: &UpdateBooking) -> Result<Booking> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_bookings::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    get_booking(pool, id)
}

pub fn delete_booking(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_bookings::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("booking"));
    }
    Ok(())
}

// ============================================================ requests ===

pub fn list_requests(pool: &DbPool) -> Result<Vec<Request>> {
    let mut c = conn(pool)?;
    Ok(marketplace_requests::table
        .order(marketplace_requests::created_at.desc())
        .load::<Request>(&mut c)?)
}

pub fn get_request(pool: &DbPool, id: &str) -> Result<Request> {
    let mut c = conn(pool)?;
    marketplace_requests::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("request"))
}

pub fn create_request(
    pool: &DbPool,
    customer_user_id: &str,
    category_id: Option<&str>,
    title: &str,
    description: Option<&str>,
    address: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    budget_min: Option<f64>,
    budget_max: Option<f64>,
    currency: &str,
    preferred_date: Option<&str>,
    expires_at: Option<chrono::NaiveDateTime>,
) -> Result<Request> {
    let id = nid();
    let new = NewRequest {
        id: &id,
        customer_user_id,
        category_id,
        title,
        description,
        address,
        latitude: latitude.map(|v| v as f32),
        longitude: longitude.map(|v| v as f32),
        budget_min: budget_min.map(|v| v as f32),
        budget_max: budget_max.map(|v| v as f32),
        currency,
        preferred_date: preferred_date.and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()),
        expires_at,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_requests::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_requests::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_request(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_requests::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("request"));
    }
    Ok(())
}

// ============================================================ bids ===

pub fn list_bids(pool: &DbPool, request_id: &str) -> Result<Vec<Bid>> {
    let mut c = conn(pool)?;
    Ok(marketplace_bids::table
        .filter(marketplace_bids::request_id.eq(request_id))
        .order(marketplace_bids::created_at.desc())
        .load::<Bid>(&mut c)?)
}

pub fn get_bid(pool: &DbPool, id: &str) -> Result<Bid> {
    let mut c = conn(pool)?;
    marketplace_bids::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("bid"))
}

pub fn create_bid(
    pool: &DbPool,
    request_id: &str,
    org_id: &str,
    price: f64,
    currency: &str,
    message: Option<&str>,
    estimated_days: Option<i32>,
) -> Result<Bid> {
    let id = nid();
    let new = NewBid {
        id: &id,
        request_id,
        org_id,
        price: price as f32,
        currency,
        message,
        estimated_days,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_bids::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_bids::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn accept_bid(pool: &DbPool, id: &str) -> Result<Bid> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_bids::table.find(id))
        .set(marketplace_bids::status.eq("accepted"))
        .execute(&mut c)?;
    // Reject other bids for same request
    let bid = get_bid(pool, id)?;
    diesel::update(
        marketplace_bids::table
            .filter(marketplace_bids::request_id.eq(&bid.request_id))
            .filter(marketplace_bids::id.ne(id)),
    )
    .set(marketplace_bids::status.eq("rejected"))
    .execute(&mut c)?;
    get_bid(pool, id)
}

pub fn delete_bid(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_bids::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("bid"));
    }
    Ok(())
}

// ============================================================ cart ===

pub fn list_cart(pool: &DbPool, user_id: &str) -> Result<Vec<CartItem>> {
    let mut c = conn(pool)?;
    Ok(marketplace_cart::table
        .filter(marketplace_cart::user_id.eq(user_id))
        .order(marketplace_cart::created_at.asc())
        .load::<CartItem>(&mut c)?)
}

pub fn add_to_cart(
    pool: &DbPool,
    user_id: &str,
    service_id: &str,
    quantity: i32,
    notes: Option<&str>,
) -> Result<CartItem> {
    // Upsert: if already in cart, increment quantity
    let existing = marketplace_cart::table
        .filter(marketplace_cart::user_id.eq(user_id))
        .filter(marketplace_cart::service_id.eq(service_id))
        .first::<CartItem>(&mut conn(pool)?)
        .ok();

    if let Some(item) = existing {
        let new_qty = item.quantity + quantity;
        let mut c = conn(pool)?;
        diesel::update(marketplace_cart::table.find(&item.id))
            .set(marketplace_cart::quantity.eq(new_qty))
            .execute(&mut c)?;
        return marketplace_cart::table
            .find(&item.id)
            .first(&mut c)
            .map_err(Into::into);
    }

    let id = nid();
    let new = NewCartItem {
        id: &id,
        user_id,
        service_id,
        quantity,
        notes,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_cart::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_cart::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_cart_item(pool: &DbPool, id: &str, patch: &UpdateCartItem) -> Result<CartItem> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_cart::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    marketplace_cart::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("cart item"))
}

pub fn remove_cart_item(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_cart::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("cart item"));
    }
    Ok(())
}

pub fn clear_cart(pool: &DbPool, user_id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(marketplace_cart::table.filter(marketplace_cart::user_id.eq(user_id)))
        .execute(&mut c)?;
    Ok(())
}

// ============================================================ reviews ===

pub fn list_reviews_by_org(pool: &DbPool, org_id: &str) -> Result<Vec<Review>> {
    let mut c = conn(pool)?;
    Ok(marketplace_reviews::table
        .filter(marketplace_reviews::org_id.eq(org_id))
        .order(marketplace_reviews::created_at.desc())
        .load::<Review>(&mut c)?)
}

pub fn create_review(
    pool: &DbPool,
    booking_id: &str,
    org_id: &str,
    service_id: Option<&str>,
    customer_user_id: &str,
    rating: i32,
    comment: Option<&str>,
) -> Result<Review> {
    let id = nid();
    let new = NewReview {
        id: &id,
        booking_id,
        org_id,
        service_id,
        customer_user_id,
        rating,
        comment,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_reviews::table)
        .values(&new)
        .execute(&mut c)?;
    // Update org rating avg/count
    let row: (Option<f64>, Option<i32>) = marketplace_reviews::table
        .filter(marketplace_reviews::org_id.eq(org_id))
        .select((
            diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Double>>(
                "AVG(rating)",
            ),
            diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Integer>>(
                "COUNT(id)",
            ),
        ))
        .first(&mut c)?;
    if let (Some(a), Some(n)) = row {
        diesel::update(marketplace_organizations::table.find(org_id))
            .set((
                marketplace_organizations::rating_avg.eq(a as f32),
                marketplace_organizations::rating_count.eq(n),
            ))
            .execute(&mut c)?;
    }
    marketplace_reviews::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

// ============================================================ commissions ===

pub fn list_commissions(pool: &DbPool, org_id: &str) -> Result<Vec<Commission>> {
    let mut c = conn(pool)?;
    Ok(marketplace_commissions::table
        .filter(marketplace_commissions::org_id.eq(org_id))
        .order(marketplace_commissions::created_at.desc())
        .load::<Commission>(&mut c)?)
}

pub fn create_commission(
    pool: &DbPool,
    booking_id: &str,
    org_id: &str,
    amount: f64,
    rate: f64,
    currency: &str,
) -> Result<Commission> {
    let id = nid();
    let new = NewCommission {
        id: &id,
        booking_id,
        org_id,
        amount: amount as f32,
        rate: rate as f32,
        currency,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_commissions::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_commissions::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

// ============================================================ payouts ===

pub fn list_payouts(pool: &DbPool, org_id: &str) -> Result<Vec<Payout>> {
    let mut c = conn(pool)?;
    Ok(marketplace_payouts::table
        .filter(marketplace_payouts::org_id.eq(org_id))
        .order(marketplace_payouts::created_at.desc())
        .load::<Payout>(&mut c)?)
}

pub fn list_all_payouts(pool: &DbPool) -> Result<Vec<Payout>> {
    let mut c = conn(pool)?;
    Ok(marketplace_payouts::table
        .order(marketplace_payouts::created_at.desc())
        .load::<Payout>(&mut c)?)
}

pub fn create_payout(
    pool: &DbPool,
    org_id: &str,
    amount: f64,
    currency: &str,
    method: &str,
    reference: Option<&str>,
) -> Result<Payout> {
    let id = nid();
    let new = NewPayout {
        id: &id,
        org_id,
        amount: amount as f32,
        currency,
        method,
        reference,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_payouts::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_payouts::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn complete_payout(pool: &DbPool, id: &str) -> Result<Payout> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_payouts::table.find(id))
        .set((
            marketplace_payouts::status.eq("completed"),
            marketplace_payouts::completed_at.eq(now()),
        ))
        .execute(&mut c)?;
    marketplace_payouts::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("payout"))
}

// ============================================================ promotions ===

pub fn list_promotions(pool: &DbPool) -> Result<Vec<Promotion>> {
    let mut c = conn(pool)?;
    Ok(marketplace_promotions::table
        .order(marketplace_promotions::created_at.desc())
        .load::<Promotion>(&mut c)?)
}

pub fn get_promotion(pool: &DbPool, id: &str) -> Result<Promotion> {
    let mut c = conn(pool)?;
    marketplace_promotions::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("promotion"))
}

pub fn validate_promo_code(pool: &DbPool, code: &str) -> Result<Promotion> {
    let mut c = conn(pool)?;
    let promo = marketplace_promotions::table
        .filter(marketplace_promotions::code.eq(code))
        .filter(marketplace_promotions::is_active.eq(1i32))
        .first::<Promotion>(&mut c)
        .map_err(|_| Error::NotFound("promotion"))?;
    let t = now();
    if promo.expires_at < t {
        return Err(Error::Invalid("promotion has expired".into()));
    }
    if let Some(max) = promo.max_uses {
        if promo.used_count >= max {
            return Err(Error::Invalid("promotion usage limit reached".into()));
        }
    }
    Ok(promo)
}

pub fn create_promotion(
    pool: &DbPool,
    org_id: Option<&str>,
    code: &str,
    description: Option<&str>,
    discount_type: &str,
    discount_value: f32,
    min_booking_value: Option<f32>,
    max_uses: Option<i32>,
    starts_at: chrono::NaiveDateTime,
    expires_at: chrono::NaiveDateTime,
) -> Result<Promotion> {
    let id = nid();
    let new = NewPromotion {
        id: &id,
        org_id,
        code,
        description,
        discount_type,
        discount_value,
        min_booking_value,
        max_uses,
        starts_at,
        expires_at,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_promotions::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_promotions::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_promotion(pool: &DbPool, id: &str, patch: &UpdatePromotion) -> Result<Promotion> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_promotions::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    get_promotion(pool, id)
}

pub fn delete_promotion(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(marketplace_promotions::table.find(id)).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("promotion"));
    }
    Ok(())
}

// ============================================================ notifications ===

pub fn list_notifications(pool: &DbPool, user_id: &str) -> Result<Vec<Notification>> {
    let mut c = conn(pool)?;
    Ok(marketplace_notifications::table
        .filter(marketplace_notifications::user_id.eq(user_id))
        .order(marketplace_notifications::created_at.desc())
        .limit(50)
        .load::<Notification>(&mut c)?)
}

pub fn mark_notification_read(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::update(marketplace_notifications::table.find(id))
        .set(marketplace_notifications::is_read.eq(1i32))
        .execute(&mut c)?;
    Ok(())
}

pub fn mark_all_notifications_read(pool: &DbPool, user_id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::update(
        marketplace_notifications::table
            .filter(marketplace_notifications::user_id.eq(user_id))
            .filter(marketplace_notifications::is_read.eq(0i32)),
    )
    .set(marketplace_notifications::is_read.eq(1i32))
    .execute(&mut c)?;
    Ok(())
}

pub fn create_notification(
    pool: &DbPool,
    user_id: &str,
    kind: &str,
    title: &str,
    body: Option<&str>,
    data: Option<&str>,
) -> Result<Notification> {
    let id = nid();
    let new = NewNotification {
        id: &id,
        user_id,
        kind,
        title,
        body,
        data,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(marketplace_notifications::table)
        .values(&new)
        .execute(&mut c)?;
    marketplace_notifications::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
