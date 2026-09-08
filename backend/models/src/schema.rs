// @generated automatically by Diesel CLI.

diesel::table! {
    accounting_invoices (id) {
        id -> Text,
        invoice_number -> Text,
        sales_order_id -> Nullable<Text>,
        quote_id -> Nullable<Text>,
        total -> Float4,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    crm_activities (id) {
        id -> Text,
        related_type -> Text,
        related_id -> Text,
        activity_type -> Text,
        subject -> Text,
        due_at -> Nullable<Timestamptz>,
        done -> Int4,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    crm_contacts (id) {
        id -> Text,
        customer_id -> Text,
        name -> Text,
        email -> Nullable<Text>,
        role -> Nullable<Text>,
        phone -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    crm_customers (id) {
        id -> Text,
        customer_number -> Text,
        name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        address -> Nullable<Text>,
        lead_id -> Nullable<Text>,
        opportunity_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    crm_leads (id) {
        id -> Text,
        lead_number -> Text,
        company_name -> Nullable<Text>,
        contact_name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        source -> Nullable<Text>,
        status -> Text,
        assigned_to -> Nullable<Text>,
        product_id -> Nullable<Text>,
        estimated_value -> Nullable<Float4>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    crm_opportunities (id) {
        id -> Text,
        opp_number -> Text,
        lead_id -> Nullable<Text>,
        customer_id -> Nullable<Text>,
        stage -> Text,
        probability -> Nullable<Float4>,
        amount -> Nullable<Float4>,
        expected_close -> Nullable<Timestamptz>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    crm_quote_lines (id) {
        id -> Text,
        quote_id -> Text,
        product_id -> Text,
        quantity -> Float4,
        unit_price -> Float4,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    crm_quotes (id) {
        id -> Text,
        quote_number -> Text,
        customer_id -> Nullable<Text>,
        opportunity_id -> Nullable<Text>,
        status -> Text,
        valid_until -> Nullable<Timestamptz>,
        total -> Nullable<Float4>,
        sales_order_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_accommodation (id) {
        id -> Text,
        name -> Text,
        destination_id -> Nullable<Text>,
        supplier_id -> Nullable<Text>,
        room_type -> Nullable<Text>,
        capacity -> Int4,
        nightly_rate -> Float8,
        #[max_length = 3]
        currency -> Bpchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_activities (id) {
        id -> Text,
        name -> Text,
        category -> Nullable<Text>,
        duration_minutes -> Int4,
        price -> Float8,
        #[max_length = 3]
        currency -> Bpchar,
        supplier_id -> Nullable<Text>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_bookings (id) {
        id -> Text,
        quote_id -> Text,
        status -> Text,
        deposit_paid -> Float8,
        balance_due -> Float8,
        #[max_length = 3]
        currency -> Bpchar,
        total -> Float8,
        confirmed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_destinations (id) {
        id -> Text,
        country -> Text,
        region -> Nullable<Text>,
        city -> Nullable<Text>,
        name -> Text,
        lat -> Nullable<Float8>,
        lon -> Nullable<Float8>,
        timezone -> Nullable<Text>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_documents (id) {
        id -> Text,
        trip_id -> Text,
        kind -> Text,
        file_id -> Nullable<Int4>,
        collection -> Text,
        filename -> Nullable<Text>,
        is_primary -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_feedback (id) {
        id -> Text,
        trip_id -> Text,
        customer_id -> Nullable<Text>,
        rating -> Int4,
        category -> Nullable<Text>,
        comment -> Nullable<Text>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_guides (id) {
        id -> Text,
        full_name -> Text,
        languages -> Nullable<Text>,
        daily_rate -> Float8,
        #[max_length = 3]
        currency -> Bpchar,
        phone -> Nullable<Text>,
        email -> Nullable<Text>,
        supplier_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_incidents (id) {
        id -> Text,
        trip_id -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        severity -> Text,
        title -> Text,
        description -> Nullable<Text>,
        assigned_to -> Nullable<Text>,
        replacement_vehicle -> Nullable<Text>,
        extra_cost -> Float4,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        resolved_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    dmc_itineraries (id) {
        id -> Text,
        package_id -> Text,
        day_number -> Int4,
        title -> Text,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    dmc_itinerary_stops (id) {
        id -> Text,
        itinerary_id -> Text,
        day_number -> Int4,
        hour -> Int4,
        location_id -> Nullable<Text>,
        activity -> Nullable<Text>,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    dmc_operations (id) {
        id -> Text,
        trip_id -> Text,
        day_number -> Int4,
        service_id -> Nullable<Text>,
        assignee_type -> Text,
        assignee_id -> Nullable<Text>,
        status -> Text,
        start_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_packages (id) {
        id -> Text,
        name -> Text,
        duration_days -> Int4,
        base_price -> Float8,
        #[max_length = 3]
        currency -> Bpchar,
        description -> Nullable<Text>,
        cover_image_file_id -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_payments (id) {
        id -> Text,
        booking_id -> Text,
        kind -> Text,
        amount -> Float4,
        currency -> Text,
        method -> Nullable<Text>,
        status -> Text,
        paid_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_quotes (id) {
        id -> Text,
        trip_id -> Text,
        version -> Int4,
        #[max_length = 3]
        currency -> Bpchar,
        subtotal -> Float8,
        tax -> Float8,
        total -> Float8,
        status -> Text,
        sent_at -> Nullable<Timestamptz>,
        accepted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_reservations (id) {
        id -> Text,
        trip_service_id -> Text,
        supplier_id -> Nullable<Text>,
        status -> Text,
        external_ref -> Nullable<Text>,
        confirmed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_supplier_categories (supplier_id, category) {
        supplier_id -> Text,
        category -> Text,
        destination_id -> Nullable<Text>,
    }
}

diesel::table! {
    dmc_supplier_requests (id) {
        id -> Text,
        trip_id -> Text,
        supplier_id -> Text,
        kind -> Text,
        qty -> Float4,
        cost -> Float4,
        selling -> Float4,
        margin -> Float4,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_transport (id) {
        id -> Text,
        mode -> Text,
        supplier_id -> Nullable<Text>,
        capacity -> Int4,
        hourly_rate -> Float8,
        #[max_length = 3]
        currency -> Bpchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_trip_events (id) {
        id -> Text,
        trip_id -> Text,
        event_type -> Text,
        scheduled_at -> Nullable<Timestamptz>,
        actual_at -> Nullable<Timestamptz>,
        status -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_trip_services (id) {
        id -> Text,
        trip_id -> Text,
        kind -> Text,
        ref_id -> Nullable<Text>,
        day_number -> Int4,
        start_time -> Nullable<Text>,
        end_time -> Nullable<Text>,
        unit_price -> Float8,
        #[max_length = 3]
        currency -> Bpchar,
        qty -> Int4,
        total -> Float8,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    dmc_trips (id) {
        id -> Text,
        name -> Text,
        package_id -> Nullable<Text>,
        lead_id -> Nullable<Text>,
        customer_id -> Nullable<Text>,
        start_date -> Timestamptz,
        end_date -> Timestamptz,
        pax_count -> Int4,
        status -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_wallet_passes (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 100]
        pass_type -> Varchar,
        #[max_length = 100]
        serial_number -> Varchar,
        #[max_length = 255]
        auth_token -> Varchar,
        #[max_length = 36]
        user_id -> Nullable<Varchar>,
        data -> Text,
        is_active -> Bool,
        last_updated -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    dmc_wallet_registrations (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 255]
        device_id -> Varchar,
        #[max_length = 100]
        pass_type -> Varchar,
        #[max_length = 100]
        serial_number -> Varchar,
        push_token -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    erp_assets (id) {
        id -> Text,
        asset_number -> Text,
        product_id -> Nullable<Text>,
        name -> Text,
        acquisition_cost -> Float4,
        acquisition_date -> Timestamptz,
        status -> Text,
        location_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_expenses (id) {
        id -> Text,
        expense_number -> Text,
        category -> Text,
        description -> Text,
        amount -> Float4,
        expense_date -> Timestamptz,
        status -> Text,
        approved_by -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_goods_receipt_lines (id) {
        id -> Text,
        receipt_id -> Text,
        product_id -> Text,
        quantity -> Float4,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    erp_goods_receipts (id) {
        id -> Text,
        receipt_number -> Text,
        po_id -> Nullable<Text>,
        warehouse_id -> Text,
        received_at -> Timestamptz,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    erp_inventory_transactions (id) {
        id -> Text,
        product_id -> Text,
        warehouse_id -> Text,
        transaction_type -> Text,
        quantity -> Float4,
        reference_type -> Nullable<Text>,
        reference_id -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    erp_procurement_lines (id) {
        id -> Text,
        request_id -> Text,
        product_id -> Text,
        quantity -> Float4,
        unit_price -> Float4,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    erp_procurement_requests (id) {
        id -> Text,
        request_number -> Text,
        supplier_id -> Nullable<Text>,
        status -> Text,
        total -> Float4,
        notes -> Nullable<Text>,
        requested_by -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_products (id) {
        id -> Text,
        sku -> Text,
        name -> Text,
        description -> Nullable<Text>,
        unit_id -> Nullable<Text>,
        product_type -> Text,
        cost_price -> Nullable<Float4>,
        selling_price -> Nullable<Float4>,
        is_active -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_purchase_order_lines (id) {
        id -> Text,
        po_id -> Text,
        product_id -> Text,
        quantity -> Float4,
        unit_price -> Float4,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    erp_purchase_orders (id) {
        id -> Text,
        po_number -> Text,
        supplier_id -> Text,
        status -> Text,
        total -> Float4,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_sales_lines (id) {
        id -> Text,
        sales_order_id -> Text,
        product_id -> Text,
        quantity -> Float4,
        unit_price -> Float4,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    erp_sales_orders (id) {
        id -> Text,
        so_number -> Text,
        customer_name -> Text,
        status -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_shipments (id) {
        id -> Text,
        sales_order_id -> Text,
        warehouse_id -> Nullable<Text>,
        shipped_at -> Timestamptz,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    erp_suppliers (id) {
        id -> Text,
        name -> Text,
        contact_email -> Nullable<Text>,
        contact_phone -> Nullable<Text>,
        address -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_units (id) {
        id -> Text,
        name -> Text,
        symbol -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    erp_warehouses (id) {
        id -> Text,
        name -> Text,
        location -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_fleet_events (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        vehicle_id -> Varchar,
        #[max_length = 50]
        event_type -> Varchar,
        lat -> Nullable<Float8>,
        lon -> Nullable<Float8>,
        data -> Text,
        recorded_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_fleet_vehicles (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 50]
        registration -> Varchar,
        #[max_length = 50]
        vehicle_type -> Varchar,
        #[max_length = 100]
        make -> Nullable<Varchar>,
        #[max_length = 100]
        model -> Nullable<Varchar>,
        year -> Nullable<Int4>,
        capacity -> Nullable<Int4>,
        #[max_length = 20]
        status -> Varchar,
        lat -> Nullable<Float8>,
        lon -> Nullable<Float8>,
        heading -> Nullable<Float8>,
        speed -> Nullable<Float8>,
        last_seen_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_password_resets (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 64]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        used_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_permissions (id) {
        id -> Int4,
        #[max_length = 64]
        module -> Varchar,
        #[max_length = 64]
        action -> Varchar,
        #[max_length = 255]
        description -> Varchar,
    }
}

diesel::table! {
    gateway_profiles (user_id) {
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 120]
        display_name -> Nullable<Varchar>,
        #[max_length = 32]
        phone -> Nullable<Varchar>,
        #[max_length = 120]
        location -> Nullable<Varchar>,
        bio -> Nullable<Text>,
        avatar_file_id -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_releases (id) {
        id -> Int4,
        #[max_length = 50]
        version -> Varchar,
        #[max_length = 32]
        platform -> Varchar,
        #[max_length = 255]
        filename -> Varchar,
        file_size -> Nullable<Int4>,
        #[max_length = 64]
        sha256 -> Nullable<Varchar>,
        changelog -> Nullable<Text>,
        download_count -> Int4,
        #[max_length = 36]
        created_by -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_revoked_tokens (jti) {
        #[max_length = 36]
        jti -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        expires_at -> Timestamptz,
        revoked_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_role_permissions (role_id, permission_id) {
        role_id -> Int4,
        permission_id -> Int4,
    }
}

diesel::table! {
    gateway_roles (id) {
        id -> Int4,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 255]
        description -> Varchar,
    }
}

diesel::table! {
    gateway_sessions (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 64]
        refresh_token_hash -> Varchar,
        #[max_length = 200]
        device -> Nullable<Varchar>,
        #[max_length = 500]
        user_agent -> Nullable<Varchar>,
        #[max_length = 64]
        ip -> Nullable<Varchar>,
        created_at -> Timestamptz,
        last_seen_at -> Timestamptz,
        expires_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    gateway_system (key) {
        #[max_length = 128]
        key -> Varchar,
        value -> Text,
        updated_at -> Timestamptz,
        #[max_length = 36]
        updated_by -> Nullable<Varchar>,
    }
}

diesel::table! {
    gateway_user_roles (user_id, role_id) {
        #[max_length = 36]
        user_id -> Varchar,
        role_id -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_users (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 320]
        email -> Varchar,
        #[max_length = 320]
        email_lower -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 120]
        display_name -> Varchar,
        #[max_length = 500]
        avatar_url -> Varchar,
        is_active -> Bool,
        token_version -> Int4,
        #[max_length = 120]
        first_name -> Varchar,
        #[max_length = 120]
        middle_name -> Varchar,
        #[max_length = 120]
        last_name -> Varchar,
        #[max_length = 60]
        username -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    gateway_vault (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 64]
        service -> Varchar,
        #[max_length = 64]
        kind -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        secret -> Text,
        meta -> Text,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    map_features (id) {
        id -> Int4,
        layer_id -> Int4,
        #[max_length = 64]
        feature_key -> Varchar,
        #[max_length = 32]
        feature_type -> Varchar,
        #[max_length = 32]
        geometry_type -> Varchar,
        geometry -> Nullable<Bytea>,
        properties -> Nullable<Text>,
        bbox_min_lon -> Nullable<Float8>,
        bbox_min_lat -> Nullable<Float8>,
        bbox_max_lon -> Nullable<Float8>,
        bbox_max_lat -> Nullable<Float8>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    map_layers (id) {
        id -> Int4,
        source_id -> Int4,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 64]
        layer_key -> Varchar,
        #[max_length = 32]
        layer_type -> Varchar,
        #[max_length = 500]
        description -> Nullable<Varchar>,
        min_zoom -> Int4,
        max_zoom -> Int4,
        z_index -> Int4,
        visible -> Bool,
        style_id -> Nullable<Int4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    map_sources (id) {
        id -> Int4,
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
    map_styles (id) {
        id -> Int4,
        layer_id -> Nullable<Int4>,
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
    map_tile_features (id) {
        id -> Int4,
        feature_id -> Int4,
        z -> Int4,
        x -> Int4,
        y -> Int4,
    }
}

diesel::table! {
    map_tiles (id) {
        id -> Int4,
        #[max_length = 64]
        layer_key -> Varchar,
        z -> Int4,
        x -> Int4,
        y -> Int4,
        data -> Nullable<Bytea>,
        #[max_length = 64]
        etag -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    marketplace_bids (id) {
        id -> Text,
        request_id -> Text,
        org_id -> Text,
        price -> Float4,
        currency -> Text,
        message -> Nullable<Text>,
        estimated_days -> Nullable<Int4>,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_bookings (id) {
        id -> Text,
        org_id -> Text,
        service_id -> Text,
        customer_user_id -> Text,
        serviceman_user_id -> Nullable<Text>,
        booking_type -> Text,
        status -> Text,
        scheduled_at -> Nullable<Timestamptz>,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        address -> Nullable<Text>,
        latitude -> Nullable<Float4>,
        longitude -> Nullable<Float4>,
        notes -> Nullable<Text>,
        total_price -> Float4,
        currency -> Text,
        commission_amount -> Float4,
        provider_payout -> Float4,
        payment_status -> Text,
        payment_method -> Nullable<Text>,
        promo_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_cart (id) {
        id -> Text,
        user_id -> Text,
        service_id -> Text,
        quantity -> Int4,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_categories (id) {
        id -> Text,
        name -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        icon_url -> Nullable<Text>,
        parent_id -> Nullable<Text>,
        sort_order -> Int4,
        is_active -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_commissions (id) {
        id -> Text,
        booking_id -> Text,
        org_id -> Text,
        amount -> Float4,
        rate -> Float4,
        currency -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_notifications (id) {
        id -> Text,
        user_id -> Text,
        kind -> Text,
        title -> Text,
        body -> Nullable<Text>,
        data -> Nullable<Text>,
        is_read -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_org_members (id) {
        id -> Text,
        org_id -> Text,
        user_id -> Text,
        role -> Text,
        is_active -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_organizations (id) {
        id -> Text,
        owner_user_id -> Text,
        name -> Text,
        slug -> Text,
        business_name -> Nullable<Text>,
        logo_url -> Nullable<Text>,
        description -> Nullable<Text>,
        phone -> Nullable<Text>,
        email -> Nullable<Text>,
        website -> Nullable<Text>,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        country -> Nullable<Text>,
        latitude -> Nullable<Float4>,
        longitude -> Nullable<Float4>,
        timezone -> Nullable<Text>,
        base_currency -> Text,
        is_verified -> Int4,
        is_active -> Int4,
        rating_avg -> Float4,
        rating_count -> Int4,
        commission_rate -> Float4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_payouts (id) {
        id -> Text,
        org_id -> Text,
        amount -> Float4,
        currency -> Text,
        method -> Text,
        reference -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    marketplace_promotions (id) {
        id -> Text,
        org_id -> Nullable<Text>,
        code -> Text,
        description -> Nullable<Text>,
        discount_type -> Text,
        discount_value -> Float4,
        min_booking_value -> Nullable<Float4>,
        max_uses -> Nullable<Int4>,
        used_count -> Int4,
        starts_at -> Timestamptz,
        expires_at -> Timestamptz,
        is_active -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_requests (id) {
        id -> Text,
        customer_user_id -> Text,
        category_id -> Nullable<Text>,
        title -> Text,
        description -> Nullable<Text>,
        address -> Nullable<Text>,
        latitude -> Nullable<Float4>,
        longitude -> Nullable<Float4>,
        budget_min -> Nullable<Float4>,
        budget_max -> Nullable<Float4>,
        currency -> Text,
        preferred_date -> Nullable<Timestamptz>,
        status -> Text,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_reviews (id) {
        id -> Text,
        booking_id -> Text,
        org_id -> Text,
        service_id -> Nullable<Text>,
        customer_user_id -> Text,
        rating -> Int4,
        comment -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_service_images (id) {
        id -> Text,
        service_id -> Text,
        url -> Text,
        sort_order -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_services (id) {
        id -> Text,
        org_id -> Text,
        category_id -> Nullable<Text>,
        name -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        short_description -> Nullable<Text>,
        cover_image_url -> Nullable<Text>,
        price_type -> Text,
        base_price -> Float4,
        currency -> Text,
        duration_minutes -> Nullable<Int4>,
        is_active -> Int4,
        rating_avg -> Float4,
        rating_count -> Int4,
        booking_count -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    marketplace_zones (id) {
        id -> Text,
        org_id -> Text,
        name -> Text,
        latitude -> Nullable<Float4>,
        longitude -> Nullable<Float4>,
        radius_km -> Nullable<Float4>,
        city -> Nullable<Text>,
        country -> Nullable<Text>,
        is_active -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    organization_subscriptions (id) {
        id -> Text,
        org_id -> Text,
        plan_id -> Text,
        status -> Text,
        current_period_start -> Timestamptz,
        current_period_end -> Timestamptz,
        cancel_at_period_end -> Int4,
        payment_method -> Nullable<Text>,
        trial_ends_at -> Nullable<Timestamptz>,
        cancelled_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    subscription_invoices (id) {
        id -> Text,
        subscription_id -> Text,
        org_id -> Text,
        amount -> Float4,
        currency -> Text,
        status -> Text,
        due_date -> Timestamptz,
        paid_at -> Nullable<Timestamptz>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    subscription_plans (id) {
        id -> Text,
        name -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        price_monthly -> Float4,
        price_yearly -> Float4,
        currency -> Text,
        max_users -> Int4,
        max_products -> Int4,
        max_orders -> Int4,
        features -> Text,
        is_active -> Int4,
        is_public -> Int4,
        sort_order -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    support_tickets (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 255]
        subject -> Varchar,
        description -> Text,
        #[max_length = 20]
        status -> Varchar,
        #[max_length = 20]
        priority -> Varchar,
        #[max_length = 64]
        category -> Varchar,
        #[max_length = 36]
        created_by -> Varchar,
        #[max_length = 36]
        assigned_to -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        closed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    thirdparty_config (provider) {
        provider -> Text,
        is_enabled -> Int4,
        config_json -> Text,
        updated_at -> Timestamptz,
        updated_by -> Nullable<Text>,
    }
}

diesel::table! {
    thirdparty_logs (id) {
        id -> Text,
        provider -> Text,
        action -> Text,
        status -> Text,
        request_data -> Nullable<Text>,
        response_data -> Nullable<Text>,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_files (id) {
        id -> Int4,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 255]
        filename -> Varchar,
        #[max_length = 255]
        original_name -> Varchar,
        #[max_length = 100]
        mime_type -> Varchar,
        size_bytes -> Int4,
        #[max_length = 500]
        storage_path -> Varchar,
        #[max_length = 64]
        collection -> Varchar,
        is_public -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    website_bookings (id) {
        id -> Text,
        template_id -> Text,
        user_id -> Nullable<Text>,
        customer_name -> Text,
        customer_email -> Nullable<Text>,
        service -> Nullable<Text>,
        product_id -> Nullable<Text>,
        date -> Timestamptz,
        status -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    website_cart_items (id) {
        id -> Text,
        template_id -> Text,
        user_id -> Text,
        product_id -> Text,
        quantity -> Float4,
        unit_price -> Float4,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    website_links (id) {
        id -> Text,
        template_id -> Text,
        label -> Text,
        href -> Text,
        icon -> Nullable<Text>,
        position -> Int4,
        is_visible -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    website_sections (id) {
        id -> Text,
        template_id -> Text,
        kind -> Text,
        position -> Int4,
        config_json -> Text,
        is_visible -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    website_templates (id) {
        id -> Text,
        slug -> Text,
        name -> Text,
        version -> Text,
        is_active -> Int4,
        theme_json -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(accounting_invoices -> crm_quotes (quote_id));
diesel::joinable!(accounting_invoices -> erp_sales_orders (sales_order_id));
diesel::joinable!(crm_contacts -> crm_customers (customer_id));
diesel::joinable!(crm_customers -> crm_leads (lead_id));
diesel::joinable!(crm_customers -> crm_opportunities (opportunity_id));
diesel::joinable!(crm_leads -> erp_products (product_id));
diesel::joinable!(crm_opportunities -> crm_leads (lead_id));
diesel::joinable!(crm_quote_lines -> crm_quotes (quote_id));
diesel::joinable!(crm_quote_lines -> erp_products (product_id));
diesel::joinable!(crm_quotes -> crm_customers (customer_id));
diesel::joinable!(crm_quotes -> crm_opportunities (opportunity_id));
diesel::joinable!(crm_quotes -> erp_sales_orders (sales_order_id));
diesel::joinable!(dmc_accommodation -> dmc_destinations (destination_id));
diesel::joinable!(dmc_accommodation -> erp_suppliers (supplier_id));
diesel::joinable!(dmc_activities -> erp_suppliers (supplier_id));
diesel::joinable!(dmc_bookings -> dmc_quotes (quote_id));
diesel::joinable!(dmc_documents -> dmc_trips (trip_id));
diesel::joinable!(dmc_documents -> user_files (file_id));
diesel::joinable!(dmc_feedback -> crm_customers (customer_id));
diesel::joinable!(dmc_feedback -> dmc_trips (trip_id));
diesel::joinable!(dmc_guides -> erp_suppliers (supplier_id));
diesel::joinable!(dmc_incidents -> dmc_trips (trip_id));
diesel::joinable!(dmc_incidents -> gateway_users (assigned_to));
diesel::joinable!(dmc_itineraries -> dmc_packages (package_id));
diesel::joinable!(dmc_itinerary_stops -> dmc_destinations (location_id));
diesel::joinable!(dmc_itinerary_stops -> dmc_itineraries (itinerary_id));
diesel::joinable!(dmc_operations -> dmc_trip_services (service_id));
diesel::joinable!(dmc_operations -> dmc_trips (trip_id));
diesel::joinable!(dmc_payments -> dmc_bookings (booking_id));
diesel::joinable!(dmc_quotes -> dmc_trips (trip_id));
diesel::joinable!(dmc_reservations -> dmc_trip_services (trip_service_id));
diesel::joinable!(dmc_reservations -> erp_suppliers (supplier_id));
diesel::joinable!(dmc_supplier_categories -> dmc_destinations (destination_id));
diesel::joinable!(dmc_supplier_categories -> erp_suppliers (supplier_id));
diesel::joinable!(dmc_supplier_requests -> dmc_trips (trip_id));
diesel::joinable!(dmc_supplier_requests -> erp_suppliers (supplier_id));
diesel::joinable!(dmc_transport -> erp_suppliers (supplier_id));
diesel::joinable!(dmc_trip_events -> dmc_trips (trip_id));
diesel::joinable!(dmc_trip_services -> dmc_trips (trip_id));
diesel::joinable!(dmc_trips -> crm_customers (customer_id));
diesel::joinable!(dmc_trips -> crm_leads (lead_id));
diesel::joinable!(dmc_trips -> dmc_packages (package_id));
diesel::joinable!(dmc_wallet_passes -> gateway_users (user_id));
diesel::joinable!(erp_assets -> erp_products (product_id));
diesel::joinable!(erp_assets -> erp_warehouses (location_id));
diesel::joinable!(erp_goods_receipt_lines -> erp_goods_receipts (receipt_id));
diesel::joinable!(erp_goods_receipt_lines -> erp_products (product_id));
diesel::joinable!(erp_goods_receipts -> erp_purchase_orders (po_id));
diesel::joinable!(erp_goods_receipts -> erp_warehouses (warehouse_id));
diesel::joinable!(erp_inventory_transactions -> erp_products (product_id));
diesel::joinable!(erp_inventory_transactions -> erp_warehouses (warehouse_id));
diesel::joinable!(erp_procurement_lines -> erp_procurement_requests (request_id));
diesel::joinable!(erp_procurement_lines -> erp_products (product_id));
diesel::joinable!(erp_procurement_requests -> erp_suppliers (supplier_id));
diesel::joinable!(erp_procurement_requests -> gateway_users (requested_by));
diesel::joinable!(erp_products -> erp_units (unit_id));
diesel::joinable!(erp_purchase_order_lines -> erp_products (product_id));
diesel::joinable!(erp_purchase_order_lines -> erp_purchase_orders (po_id));
diesel::joinable!(erp_purchase_orders -> erp_suppliers (supplier_id));
diesel::joinable!(erp_sales_lines -> erp_products (product_id));
diesel::joinable!(erp_sales_lines -> erp_sales_orders (sales_order_id));
diesel::joinable!(erp_shipments -> erp_sales_orders (sales_order_id));
diesel::joinable!(erp_shipments -> erp_warehouses (warehouse_id));
diesel::joinable!(gateway_fleet_events -> gateway_fleet_vehicles (vehicle_id));
diesel::joinable!(gateway_password_resets -> gateway_users (user_id));
diesel::joinable!(gateway_profiles -> gateway_users (user_id));
diesel::joinable!(gateway_releases -> gateway_users (created_by));
diesel::joinable!(gateway_revoked_tokens -> gateway_users (user_id));
diesel::joinable!(gateway_role_permissions -> gateway_permissions (permission_id));
diesel::joinable!(gateway_role_permissions -> gateway_roles (role_id));
diesel::joinable!(gateway_sessions -> gateway_users (user_id));
diesel::joinable!(gateway_system -> gateway_users (updated_by));
diesel::joinable!(gateway_user_roles -> gateway_roles (role_id));
diesel::joinable!(gateway_user_roles -> gateway_users (user_id));
diesel::joinable!(gateway_vault -> gateway_users (user_id));
diesel::joinable!(map_features -> map_layers (layer_id));
diesel::joinable!(map_layers -> map_sources (source_id));
diesel::joinable!(map_styles -> map_layers (layer_id));
diesel::joinable!(map_tile_features -> map_features (feature_id));
diesel::joinable!(marketplace_bids -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_bids -> marketplace_requests (request_id));
diesel::joinable!(marketplace_bookings -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_bookings -> marketplace_services (service_id));
diesel::joinable!(marketplace_cart -> gateway_users (user_id));
diesel::joinable!(marketplace_cart -> marketplace_services (service_id));
diesel::joinable!(marketplace_commissions -> marketplace_bookings (booking_id));
diesel::joinable!(marketplace_commissions -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_notifications -> gateway_users (user_id));
diesel::joinable!(marketplace_org_members -> gateway_users (user_id));
diesel::joinable!(marketplace_org_members -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_organizations -> gateway_users (owner_user_id));
diesel::joinable!(marketplace_payouts -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_promotions -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_requests -> gateway_users (customer_user_id));
diesel::joinable!(marketplace_requests -> marketplace_categories (category_id));
diesel::joinable!(marketplace_reviews -> gateway_users (customer_user_id));
diesel::joinable!(marketplace_reviews -> marketplace_bookings (booking_id));
diesel::joinable!(marketplace_reviews -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_reviews -> marketplace_services (service_id));
diesel::joinable!(marketplace_service_images -> marketplace_services (service_id));
diesel::joinable!(marketplace_services -> marketplace_categories (category_id));
diesel::joinable!(marketplace_services -> marketplace_organizations (org_id));
diesel::joinable!(marketplace_zones -> marketplace_organizations (org_id));
diesel::joinable!(organization_subscriptions -> marketplace_organizations (org_id));
diesel::joinable!(organization_subscriptions -> subscription_plans (plan_id));
diesel::joinable!(subscription_invoices -> marketplace_organizations (org_id));
diesel::joinable!(subscription_invoices -> organization_subscriptions (subscription_id));
diesel::joinable!(thirdparty_config -> gateway_users (updated_by));
diesel::joinable!(user_files -> gateway_users (user_id));
diesel::joinable!(website_bookings -> erp_products (product_id));
diesel::joinable!(website_bookings -> gateway_users (user_id));
diesel::joinable!(website_bookings -> website_templates (template_id));
diesel::joinable!(website_cart_items -> erp_products (product_id));
diesel::joinable!(website_cart_items -> gateway_users (user_id));
diesel::joinable!(website_cart_items -> website_templates (template_id));
diesel::joinable!(website_links -> website_templates (template_id));
diesel::joinable!(website_sections -> website_templates (template_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounting_invoices,
    crm_activities,
    crm_contacts,
    crm_customers,
    crm_leads,
    crm_opportunities,
    crm_quote_lines,
    crm_quotes,
    dmc_accommodation,
    dmc_activities,
    dmc_bookings,
    dmc_destinations,
    dmc_documents,
    dmc_feedback,
    dmc_guides,
    dmc_incidents,
    dmc_itineraries,
    dmc_itinerary_stops,
    dmc_operations,
    dmc_packages,
    dmc_payments,
    dmc_quotes,
    dmc_reservations,
    dmc_supplier_categories,
    dmc_supplier_requests,
    dmc_transport,
    dmc_trip_events,
    dmc_trip_services,
    dmc_trips,
    dmc_wallet_passes,
    dmc_wallet_registrations,
    erp_assets,
    erp_expenses,
    erp_goods_receipt_lines,
    erp_goods_receipts,
    erp_inventory_transactions,
    erp_procurement_lines,
    erp_procurement_requests,
    erp_products,
    erp_purchase_order_lines,
    erp_purchase_orders,
    erp_sales_lines,
    erp_sales_orders,
    erp_shipments,
    erp_suppliers,
    erp_units,
    erp_warehouses,
    gateway_fleet_events,
    gateway_fleet_vehicles,
    gateway_password_resets,
    gateway_permissions,
    gateway_profiles,
    gateway_releases,
    gateway_revoked_tokens,
    gateway_role_permissions,
    gateway_roles,
    gateway_sessions,
    gateway_system,
    gateway_user_roles,
    gateway_users,
    gateway_vault,
    map_features,
    map_layers,
    map_sources,
    map_styles,
    map_tile_features,
    map_tiles,
    marketplace_bids,
    marketplace_bookings,
    marketplace_cart,
    marketplace_categories,
    marketplace_commissions,
    marketplace_notifications,
    marketplace_org_members,
    marketplace_organizations,
    marketplace_payouts,
    marketplace_promotions,
    marketplace_requests,
    marketplace_reviews,
    marketplace_service_images,
    marketplace_services,
    marketplace_zones,
    organization_subscriptions,
    subscription_invoices,
    subscription_plans,
    support_tickets,
    thirdparty_config,
    thirdparty_logs,
    user_files,
    website_bookings,
    website_cart_items,
    website_links,
    website_sections,
    website_templates,
);
