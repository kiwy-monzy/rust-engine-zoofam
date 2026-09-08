diesel::table! {
    map_sources (id) {
        id -> Integer,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 32]
        source_type -> Varchar,
        #[max_length = 255]
        url -> Nullable<Varchar>,
        #[max_length = 64]
        version -> Nullable<Varchar>,
        #[max_length = 500]
        description -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    map_layers (id) {
        id -> Integer,
        source_id -> Integer,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 64]
        layer_key -> Varchar,
        #[max_length = 32]
        layer_type -> Varchar,
        #[max_length = 500]
        description -> Nullable<Varchar>,
        min_zoom -> Integer,
        max_zoom -> Integer,
        z_index -> Integer,
        visible -> Bool,
        style_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    map_features (id) {
        id -> Integer,
        layer_id -> Integer,
        #[max_length = 64]
        feature_key -> Varchar,
        #[max_length = 32]
        feature_type -> Varchar,
        #[max_length = 32]
        geometry_type -> Varchar,
        geometry -> Binary,
        #[max_length = 2000]
        properties -> Nullable<Text>,
        bbox_min_lon -> Nullable<Double>,
        bbox_min_lat -> Nullable<Double>,
        bbox_max_lon -> Nullable<Double>,
        bbox_max_lat -> Nullable<Double>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    map_styles (id) {
        id -> Integer,
        layer_id -> Nullable<Integer>,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 32]
        style_type -> Varchar,
        definition -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    map_tiles (id) {
        id -> Integer,
        #[max_length = 64]
        layer_key -> Varchar,
        z -> Integer,
        x -> Integer,
        y -> Integer,
        data -> Binary,
        #[max_length = 64]
        etag -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    map_tile_features (id) {
        id -> Integer,
        feature_id -> Integer,
        z -> Integer,
        x -> Integer,
        y -> Integer,
    }
}

diesel::joinable!(map_layers -> map_sources (source_id));
diesel::joinable!(map_layers -> map_styles (style_id));
diesel::joinable!(map_features -> map_layers (layer_id));
diesel::joinable!(map_tile_features -> map_features (feature_id));

diesel::allow_tables_to_appear_in_same_query!(
    map_sources,
    map_layers,
    map_features,
    map_styles,
    map_tiles,
    map_tile_features,
);
