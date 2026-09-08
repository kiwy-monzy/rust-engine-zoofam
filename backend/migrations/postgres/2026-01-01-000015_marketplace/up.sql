-- ================================================================
-- Marketplace (Fundi) - organizations, services, bookings
-- ================================================================

CREATE TABLE marketplace_organizations (
    id              TEXT PRIMARY KEY,
    owner_user_id   TEXT NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    slug            TEXT UNIQUE NOT NULL,
    business_name   TEXT,
    logo_url        TEXT,
    description     TEXT,
    phone           TEXT,
    email           TEXT,
    website         TEXT,
    address         TEXT,
    city            TEXT,
    country         TEXT,
    latitude        REAL,
    longitude       REAL,
    timezone        TEXT,
    base_currency   TEXT NOT NULL DEFAULT 'TZS',
    is_verified     INTEGER NOT NULL DEFAULT 0,
    is_active       INTEGER NOT NULL DEFAULT 1,
    rating_avg      REAL NOT NULL DEFAULT 0.0,
    rating_count    INTEGER NOT NULL DEFAULT 0,
    commission_rate REAL NOT NULL DEFAULT 0.10,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_orgs_owner_idx ON marketplace_organizations (owner_user_id);
CREATE INDEX marketplace_orgs_slug_idx ON marketplace_organizations (slug);
CREATE INDEX marketplace_orgs_city_idx ON marketplace_organizations (city);

CREATE TABLE marketplace_org_members (
    id          TEXT PRIMARY KEY,
    org_id      TEXT NOT NULL REFERENCES marketplace_organizations(id) ON DELETE CASCADE,
    user_id     TEXT NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    role        TEXT NOT NULL DEFAULT 'serviceman',
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(org_id, user_id)
);
CREATE INDEX marketplace_org_members_org_idx ON marketplace_org_members (org_id);
CREATE INDEX marketplace_org_members_user_idx ON marketplace_org_members (user_id);

CREATE TABLE marketplace_categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    slug        TEXT UNIQUE NOT NULL,
    description TEXT,
    icon_url    TEXT,
    parent_id   TEXT REFERENCES marketplace_categories(id),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE marketplace_services (
    id                TEXT PRIMARY KEY,
    org_id            TEXT NOT NULL REFERENCES marketplace_organizations(id) ON DELETE CASCADE,
    category_id       TEXT REFERENCES marketplace_categories(id),
    name              TEXT NOT NULL,
    slug              TEXT NOT NULL,
    description       TEXT,
    short_description TEXT,
    cover_image_url   TEXT,
    price_type        TEXT NOT NULL DEFAULT 'fixed',
    base_price        REAL NOT NULL DEFAULT 0,
    currency          TEXT NOT NULL DEFAULT 'TZS',
    duration_minutes  INTEGER,
    is_active         INTEGER NOT NULL DEFAULT 1,
    rating_avg        REAL NOT NULL DEFAULT 0.0,
    rating_count      INTEGER NOT NULL DEFAULT 0,
    booking_count     INTEGER NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(org_id, slug)
);
CREATE INDEX marketplace_services_org_idx ON marketplace_services (org_id);
CREATE INDEX marketplace_services_category_idx ON marketplace_services (category_id);

CREATE TABLE marketplace_zones (
    id          TEXT PRIMARY KEY,
    org_id      TEXT NOT NULL REFERENCES marketplace_organizations(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    latitude    REAL,
    longitude   REAL,
    radius_km   REAL,
    city        TEXT,
    country     TEXT,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_zones_org_idx ON marketplace_zones (org_id);

CREATE TABLE marketplace_service_images (
    id          TEXT PRIMARY KEY,
    service_id  TEXT NOT NULL REFERENCES marketplace_services(id) ON DELETE CASCADE,
    url         TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_service_images_svc_idx ON marketplace_service_images (service_id);

CREATE TABLE marketplace_bookings (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT NOT NULL REFERENCES marketplace_organizations(id),
    service_id          TEXT NOT NULL REFERENCES marketplace_services(id),
    customer_user_id    TEXT NOT NULL REFERENCES gateway_users(id),
    serviceman_user_id  TEXT REFERENCES gateway_users(id),
    booking_type        TEXT NOT NULL DEFAULT 'direct',
    status              TEXT NOT NULL DEFAULT 'pending',
    scheduled_at        TIMESTAMPTZ,
    started_at          TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    address             TEXT,
    latitude            REAL,
    longitude           REAL,
    notes               TEXT,
    total_price         REAL NOT NULL DEFAULT 0,
    currency            TEXT NOT NULL DEFAULT 'TZS',
    commission_amount   REAL NOT NULL DEFAULT 0,
    provider_payout     REAL NOT NULL DEFAULT 0,
    payment_status      TEXT NOT NULL DEFAULT 'unpaid',
    payment_method      TEXT,
    promo_id            TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_bookings_org_idx ON marketplace_bookings (org_id);
CREATE INDEX marketplace_bookings_customer_idx ON marketplace_bookings (customer_user_id);
CREATE INDEX marketplace_bookings_service_idx ON marketplace_bookings (service_id);
CREATE INDEX marketplace_bookings_status_idx ON marketplace_bookings (status);

CREATE TABLE marketplace_requests (
    id                  TEXT PRIMARY KEY,
    customer_user_id    TEXT NOT NULL REFERENCES gateway_users(id),
    category_id         TEXT REFERENCES marketplace_categories(id),
    title               TEXT NOT NULL,
    description         TEXT,
    address             TEXT,
    latitude            REAL,
    longitude           REAL,
    budget_min          REAL,
    budget_max          REAL,
    currency            TEXT NOT NULL DEFAULT 'TZS',
    preferred_date      TIMESTAMPTZ,
    status              TEXT NOT NULL DEFAULT 'open',
    expires_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_requests_customer_idx ON marketplace_requests (customer_user_id);
CREATE INDEX marketplace_requests_status_idx ON marketplace_requests (status);

CREATE TABLE marketplace_bids (
    id              TEXT PRIMARY KEY,
    request_id      TEXT NOT NULL REFERENCES marketplace_requests(id) ON DELETE CASCADE,
    org_id          TEXT NOT NULL REFERENCES marketplace_organizations(id),
    price           REAL NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'TZS',
    message         TEXT,
    estimated_days  INTEGER,
    status          TEXT NOT NULL DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(request_id, org_id)
);
CREATE INDEX marketplace_bids_request_idx ON marketplace_bids (request_id);

CREATE TABLE marketplace_cart (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    service_id  TEXT NOT NULL REFERENCES marketplace_services(id) ON DELETE CASCADE,
    quantity    INTEGER NOT NULL DEFAULT 1,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, service_id)
);
CREATE INDEX marketplace_cart_user_idx ON marketplace_cart (user_id);

CREATE TABLE marketplace_reviews (
    id                  TEXT PRIMARY KEY,
    booking_id          TEXT NOT NULL REFERENCES marketplace_bookings(id),
    org_id              TEXT NOT NULL REFERENCES marketplace_organizations(id),
    service_id          TEXT REFERENCES marketplace_services(id),
    customer_user_id    TEXT NOT NULL REFERENCES gateway_users(id),
    rating              INTEGER NOT NULL,
    comment             TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_reviews_org_idx ON marketplace_reviews (org_id);
CREATE INDEX marketplace_reviews_service_idx ON marketplace_reviews (service_id);
CREATE INDEX marketplace_reviews_booking_idx ON marketplace_reviews (booking_id);

CREATE TABLE marketplace_commissions (
    id          TEXT PRIMARY KEY,
    booking_id  TEXT NOT NULL REFERENCES marketplace_bookings(id),
    org_id      TEXT NOT NULL REFERENCES marketplace_organizations(id),
    amount      REAL NOT NULL,
    rate        REAL NOT NULL,
    currency    TEXT NOT NULL DEFAULT 'TZS',
    status      TEXT NOT NULL DEFAULT 'pending',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_commissions_org_idx ON marketplace_commissions (org_id);

CREATE TABLE marketplace_payouts (
    id          TEXT PRIMARY KEY,
    org_id      TEXT NOT NULL REFERENCES marketplace_organizations(id),
    amount      REAL NOT NULL,
    currency    TEXT NOT NULL DEFAULT 'TZS',
    method      TEXT NOT NULL DEFAULT 'bank',
    reference   TEXT,
    status      TEXT NOT NULL DEFAULT 'pending',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);
CREATE INDEX marketplace_payouts_org_idx ON marketplace_payouts (org_id);

CREATE TABLE marketplace_promotions (
    id                TEXT PRIMARY KEY,
    org_id            TEXT REFERENCES marketplace_organizations(id),
    code              TEXT UNIQUE NOT NULL,
    description       TEXT,
    discount_type     TEXT NOT NULL DEFAULT 'percentage',
    discount_value    REAL NOT NULL,
    min_booking_value REAL,
    max_uses          INTEGER,
    used_count        INTEGER NOT NULL DEFAULT 0,
    starts_at         TIMESTAMPTZ NOT NULL,
    expires_at         TIMESTAMPTZ NOT NULL,
    is_active         INTEGER NOT NULL DEFAULT 1,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_promos_code_idx ON marketplace_promotions (code);

CREATE TABLE marketplace_notifications (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT,
    data        TEXT,
    is_read     INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX marketplace_notif_user_idx ON marketplace_notifications (user_id);
CREATE INDEX marketplace_notif_read_idx ON marketplace_notifications (user_id, is_read);
