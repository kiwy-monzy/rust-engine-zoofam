-- ================================================================
-- Fleet - vehicles, live tracking
-- ================================================================

CREATE TABLE gateway_fleet_vehicles (
    id              VARCHAR(36)  PRIMARY KEY,
    registration    VARCHAR(50)  NOT NULL UNIQUE,
    vehicle_type    VARCHAR(50)  NOT NULL DEFAULT 'bus',
    make            VARCHAR(100),
    model           VARCHAR(100),
    year            INTEGER,
    capacity        INTEGER,
    status          VARCHAR(20)  NOT NULL DEFAULT 'active',
    lat             DOUBLE PRECISION,
    lon             DOUBLE PRECISION,
    heading         DOUBLE PRECISION,
    speed           DOUBLE PRECISION,
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE INDEX gateway_fleet_vehicles_registration_idx ON gateway_fleet_vehicles (registration);
CREATE INDEX gateway_fleet_vehicles_status_idx ON gateway_fleet_vehicles (status);
CREATE INDEX gateway_fleet_vehicles_type_idx ON gateway_fleet_vehicles (vehicle_type);

CREATE TABLE gateway_fleet_events (
    id              VARCHAR(36)  PRIMARY KEY,
    vehicle_id      VARCHAR(36)  NOT NULL REFERENCES gateway_fleet_vehicles(id) ON DELETE CASCADE,
    event_type      VARCHAR(50)  NOT NULL,
    lat             DOUBLE PRECISION,
    lon             DOUBLE PRECISION,
    data            TEXT         NOT NULL DEFAULT '{}',
    recorded_at     TIMESTAMPTZ    NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE INDEX gateway_fleet_events_vehicle_id_idx ON gateway_fleet_events (vehicle_id);
CREATE INDEX gateway_fleet_events_type_idx ON gateway_fleet_events (event_type);
CREATE INDEX gateway_fleet_events_recorded_at_idx ON gateway_fleet_events (recorded_at DESC);
