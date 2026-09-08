-- ================================================================
-- Maps - sources, layers, styles, features, tiles
-- Matches crates/maps/src/schema.rs
-- ================================================================

CREATE TABLE map_sources (
    id              SERIAL PRIMARY KEY,
    name            VARCHAR(64) NOT NULL UNIQUE,
    source_type     VARCHAR(32) NOT NULL DEFAULT 'vector',
    url             VARCHAR(255),
    version         VARCHAR(64),
    description     VARCHAR(500),
    created_at      TIMESTAMP NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE map_layers (
    id              SERIAL PRIMARY KEY,
    source_id       INTEGER NOT NULL REFERENCES map_sources(id) ON DELETE CASCADE,
    name            VARCHAR(64) NOT NULL,
    layer_key       VARCHAR(64) NOT NULL UNIQUE,
    layer_type      VARCHAR(32) NOT NULL DEFAULT 'fill',
    description     VARCHAR(500),
    min_zoom        INTEGER NOT NULL DEFAULT 0,
    max_zoom        INTEGER NOT NULL DEFAULT 22,
    z_index         INTEGER NOT NULL DEFAULT 0,
    visible         BOOLEAN NOT NULL DEFAULT TRUE,
    style_id        INTEGER,
    created_at      TIMESTAMP NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX map_layers_source_id_idx ON map_layers (source_id);

CREATE TABLE map_styles (
    id              SERIAL PRIMARY KEY,
    layer_id        INTEGER REFERENCES map_layers(id) ON DELETE CASCADE,
    name            VARCHAR(64) NOT NULL,
    style_type      VARCHAR(32) NOT NULL DEFAULT 'fill',
    definition      TEXT NOT NULL DEFAULT '{}',
    created_at      TIMESTAMP NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX map_styles_layer_id_idx ON map_styles (layer_id);

CREATE TABLE map_features (
    id              SERIAL PRIMARY KEY,
    layer_id        INTEGER NOT NULL REFERENCES map_layers(id) ON DELETE CASCADE,
    feature_key     VARCHAR(64) NOT NULL,
    feature_type    VARCHAR(32) NOT NULL,
    geometry_type   VARCHAR(32) NOT NULL,
    geometry        BYTEA,
    properties      TEXT,
    bbox_min_lon   DOUBLE PRECISION,
    bbox_min_lat   DOUBLE PRECISION,
    bbox_max_lon   DOUBLE PRECISION,
    bbox_max_lat   DOUBLE PRECISION,
    created_at      TIMESTAMP NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX map_features_layer_id_idx ON map_features (layer_id);
CREATE INDEX map_features_bbox_idx ON map_features (bbox_min_lon, bbox_min_lat, bbox_max_lon, bbox_max_lat);

CREATE TABLE map_tiles (
    id              SERIAL PRIMARY KEY,
    layer_key       VARCHAR(64) NOT NULL,
    z               INTEGER NOT NULL,
    x               INTEGER NOT NULL,
    y               INTEGER NOT NULL,
    data            BYTEA,
    etag            VARCHAR(64),
    created_at      TIMESTAMP NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE (layer_key, z, x, y)
);
CREATE INDEX map_tiles_layer_zxy_idx ON map_tiles (layer_key, z, x, y);

CREATE TABLE map_tile_features (
    id              SERIAL PRIMARY KEY,
    feature_id      INTEGER NOT NULL REFERENCES map_features(id) ON DELETE CASCADE,
    z               INTEGER NOT NULL,
    x               INTEGER NOT NULL,
    y               INTEGER NOT NULL
);
CREATE INDEX map_tile_features_feature_idx ON map_tile_features (feature_id);
CREATE INDEX map_tile_features_xyz_idx ON map_tile_features (z, x, y);
