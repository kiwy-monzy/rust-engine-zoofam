-- ================================================================
-- DMC (Destination Management Company) - tour operator
-- ================================================================

CREATE TABLE dmc_destinations (
    id              TEXT PRIMARY KEY,
    country         TEXT NOT NULL,
    region          TEXT,
    city            TEXT,
    name            TEXT NOT NULL,
    lat             DOUBLE PRECISION,
    lon             DOUBLE PRECISION,
    timezone        TEXT,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_destinations_country_idx ON dmc_destinations (country);
CREATE INDEX dmc_destinations_city_idx ON dmc_destinations (city);

CREATE TABLE dmc_packages (
    id                       TEXT PRIMARY KEY,
    name                     TEXT NOT NULL,
    duration_days            INTEGER NOT NULL DEFAULT 1,
    base_price               DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency                 CHAR(3) NOT NULL DEFAULT 'TZS',
    description              TEXT,
    cover_image_file_id      INTEGER,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_packages_name_idx ON dmc_packages (name);

CREATE TABLE dmc_itineraries (
    id            TEXT PRIMARY KEY,
    package_id    TEXT NOT NULL REFERENCES dmc_packages(id) ON DELETE CASCADE,
    day_number    INTEGER NOT NULL,
    title         TEXT NOT NULL,
    notes         TEXT
);
CREATE INDEX dmc_itineraries_package_idx ON dmc_itineraries (package_id);

CREATE TABLE dmc_itinerary_stops (
    id            TEXT PRIMARY KEY,
    itinerary_id  TEXT NOT NULL REFERENCES dmc_itineraries(id) ON DELETE CASCADE,
    day_number    INTEGER NOT NULL,
    hour          INTEGER NOT NULL DEFAULT 8,
    location_id   TEXT REFERENCES dmc_destinations(id) ON DELETE SET NULL,
    activity      TEXT,
    notes         TEXT
);
CREATE INDEX dmc_itinerary_stops_itinerary_idx ON dmc_itinerary_stops (itinerary_id);

CREATE TABLE dmc_activities (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    category        TEXT,
    duration_minutes INTEGER NOT NULL DEFAULT 60,
    price           DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency        CHAR(3) NOT NULL DEFAULT 'TZS',
    supplier_id     TEXT REFERENCES erp_suppliers(id) ON DELETE SET NULL,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_activities_category_idx ON dmc_activities (category);

CREATE TABLE dmc_accommodation (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    destination_id  TEXT REFERENCES dmc_destinations(id) ON DELETE SET NULL,
    supplier_id     TEXT REFERENCES erp_suppliers(id) ON DELETE SET NULL,
    room_type       TEXT,
    capacity        INTEGER NOT NULL DEFAULT 1,
    nightly_rate    DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency        CHAR(3) NOT NULL DEFAULT 'TZS',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_accommodation_destination_idx ON dmc_accommodation (destination_id);

CREATE TABLE dmc_transport (
    id              TEXT PRIMARY KEY,
    mode            TEXT NOT NULL,
    supplier_id     TEXT REFERENCES erp_suppliers(id) ON DELETE SET NULL,
    capacity        INTEGER NOT NULL DEFAULT 1,
    hourly_rate     DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency        CHAR(3) NOT NULL DEFAULT 'TZS',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_transport_mode_idx ON dmc_transport (mode);

CREATE TABLE dmc_supplier_categories (
    supplier_id     TEXT NOT NULL REFERENCES erp_suppliers(id) ON DELETE CASCADE,
    category        TEXT NOT NULL,
    destination_id  TEXT REFERENCES dmc_destinations(id) ON DELETE SET NULL,
    PRIMARY KEY (supplier_id, category)
);
CREATE INDEX dmc_supplier_categories_category_idx ON dmc_supplier_categories (category);

CREATE TABLE dmc_guides (
    id              TEXT PRIMARY KEY,
    full_name       TEXT NOT NULL,
    languages       TEXT,
    daily_rate      DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency        CHAR(3) NOT NULL DEFAULT 'TZS',
    phone           TEXT,
    email           TEXT,
    supplier_id     TEXT REFERENCES erp_suppliers(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_guides_name_idx ON dmc_guides (full_name);

CREATE TABLE dmc_trips (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    package_id      TEXT REFERENCES dmc_packages(id) ON DELETE SET NULL,
    lead_id         TEXT REFERENCES crm_leads(id) ON DELETE SET NULL,
    customer_id     TEXT REFERENCES crm_customers(id) ON DELETE SET NULL,
    start_date      TIMESTAMPTZ NOT NULL,
    end_date        TIMESTAMPTZ NOT NULL,
    pax_count       INTEGER NOT NULL DEFAULT 1,
    status          TEXT NOT NULL DEFAULT 'draft',
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_trips_status_idx ON dmc_trips (status);
CREATE INDEX dmc_trips_start_date_idx ON dmc_trips (start_date);
CREATE INDEX dmc_trips_customer_idx ON dmc_trips (customer_id);
CREATE INDEX dmc_trips_package_idx ON dmc_trips (package_id);

CREATE TABLE dmc_trip_services (
    id            TEXT PRIMARY KEY,
    trip_id       TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    kind          TEXT NOT NULL,
    ref_id        TEXT,
    day_number    INTEGER NOT NULL,
    start_time    TEXT,
    end_time      TEXT,
    unit_price    DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency      CHAR(3) NOT NULL DEFAULT 'TZS',
    qty           INTEGER NOT NULL DEFAULT 1,
    total         DOUBLE PRECISION NOT NULL DEFAULT 0,
    notes         TEXT
);
CREATE INDEX dmc_trip_services_trip_idx ON dmc_trip_services (trip_id);
CREATE INDEX dmc_trip_services_kind_idx ON dmc_trip_services (kind);

CREATE TABLE dmc_quotes (
    id            TEXT PRIMARY KEY,
    trip_id       TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    version       INTEGER NOT NULL DEFAULT 1,
    currency      CHAR(3) NOT NULL DEFAULT 'TZS',
    subtotal      DOUBLE PRECISION NOT NULL DEFAULT 0,
    tax           DOUBLE PRECISION NOT NULL DEFAULT 0,
    total         DOUBLE PRECISION NOT NULL DEFAULT 0,
    status        TEXT NOT NULL DEFAULT 'draft',
    sent_at       TIMESTAMPTZ,
    accepted_at   TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_quotes_trip_idx ON dmc_quotes (trip_id);

CREATE TABLE dmc_bookings (
    id              TEXT PRIMARY KEY,
    quote_id        TEXT NOT NULL REFERENCES dmc_quotes(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'pending',
    deposit_paid    DOUBLE PRECISION NOT NULL DEFAULT 0,
    balance_due     DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency        CHAR(3) NOT NULL DEFAULT 'TZS',
    total           DOUBLE PRECISION NOT NULL DEFAULT 0,
    confirmed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_bookings_quote_idx ON dmc_bookings (quote_id);

CREATE TABLE dmc_incidents (
    id              TEXT PRIMARY KEY,
    trip_id         TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    type            TEXT NOT NULL,
    severity        TEXT NOT NULL DEFAULT 'medium',
    title           TEXT NOT NULL DEFAULT '',
    description     TEXT,
    assigned_to     TEXT REFERENCES gateway_users(id) ON DELETE SET NULL,
    replacement_vehicle TEXT,
    extra_cost      REAL NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'OPEN',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at     TIMESTAMPTZ
);
CREATE INDEX dmc_incidents_trip ON dmc_incidents (trip_id);
CREATE INDEX dmc_incidents_status ON dmc_incidents (status);

CREATE TABLE dmc_documents (
    id              TEXT PRIMARY KEY,
    trip_id         TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL DEFAULT 'voucher',
    file_id         INTEGER REFERENCES user_files(id) ON DELETE SET NULL,
    collection      TEXT NOT NULL DEFAULT 'dmc-documents',
    filename        TEXT,
    is_primary      INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_documents_trip ON dmc_documents (trip_id);

CREATE TABLE dmc_reservations (
    id              TEXT PRIMARY KEY,
    trip_service_id TEXT NOT NULL REFERENCES dmc_trip_services(id) ON DELETE CASCADE,
    supplier_id     TEXT REFERENCES erp_suppliers(id) ON DELETE SET NULL,
    status          TEXT NOT NULL DEFAULT 'PENDING',
    external_ref    TEXT,
    confirmed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_reservations_service ON dmc_reservations (trip_service_id);

CREATE TABLE dmc_supplier_requests (
    id              TEXT PRIMARY KEY,
    trip_id         TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    supplier_id     TEXT NOT NULL REFERENCES erp_suppliers(id),
    kind            TEXT NOT NULL DEFAULT 'service',
    qty             REAL NOT NULL DEFAULT 1,
    cost            REAL NOT NULL DEFAULT 0,
    selling         REAL NOT NULL DEFAULT 0,
    margin          REAL NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'REQUESTED',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_supplier_requests_trip ON dmc_supplier_requests (trip_id);

CREATE TABLE dmc_payments (
    id              TEXT PRIMARY KEY,
    booking_id      TEXT NOT NULL REFERENCES dmc_bookings(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL DEFAULT 'DEPOSIT',
    amount          REAL NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'TZS',
    method          TEXT,
    status          TEXT NOT NULL DEFAULT 'PENDING',
    paid_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_payments_booking ON dmc_payments (booking_id);

CREATE TABLE dmc_operations (
    id              TEXT PRIMARY KEY,
    trip_id         TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    day_number      INTEGER NOT NULL DEFAULT 1,
    service_id      TEXT REFERENCES dmc_trip_services(id) ON DELETE SET NULL,
    assignee_type   TEXT NOT NULL DEFAULT 'GUIDE',
    assignee_id     TEXT,
    status          TEXT NOT NULL DEFAULT 'PLANNED',
    start_at        TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE dmc_trip_events (
    id              TEXT PRIMARY KEY,
    trip_id         TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    event_type      TEXT NOT NULL,
    scheduled_at    TIMESTAMPTZ,
    actual_at       TIMESTAMPTZ,
    status          TEXT NOT NULL DEFAULT 'PLANNED',
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dmc_ops_trip_day ON dmc_operations (trip_id, day_number);
CREATE INDEX dmc_events_trip ON dmc_trip_events (trip_id);

CREATE TABLE dmc_feedback (
    id              TEXT PRIMARY KEY,
    trip_id         TEXT NOT NULL REFERENCES dmc_trips(id) ON DELETE CASCADE,
    customer_id     TEXT REFERENCES crm_customers(id) ON DELETE SET NULL,
    rating          INTEGER NOT NULL DEFAULT 5,
    category        TEXT,
    comment         TEXT,
    resolved_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
