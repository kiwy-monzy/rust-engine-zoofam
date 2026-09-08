-- ================================================================
-- Website templates (Knowlia + sub-modules)
-- ================================================================

CREATE TABLE website_templates (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    theme_json TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE website_sections (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES website_templates(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    config_json TEXT NOT NULL DEFAULT '{}',
    is_visible INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX website_sections_tid_kind ON website_sections(template_id, kind);
CREATE UNIQUE INDEX website_sections_tid_kind_uniq ON website_sections(template_id, kind);

CREATE TABLE website_links (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES website_templates(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    href TEXT NOT NULL,
    icon TEXT,
    position INTEGER NOT NULL DEFAULT 0,
    is_visible INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX website_links_tid_pos ON website_links(template_id, position);

CREATE TABLE website_cart_items (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES website_templates(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    quantity REAL NOT NULL DEFAULT 1,
    unit_price REAL NOT NULL DEFAULT 0,
    added_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX website_cart_user_tid ON website_cart_items(user_id, template_id);
CREATE INDEX website_cart_product ON website_cart_items(product_id);

CREATE TABLE website_bookings (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES website_templates(id) ON DELETE CASCADE,
    user_id TEXT REFERENCES gateway_users(id) ON DELETE SET NULL,
    customer_name TEXT NOT NULL,
    customer_email TEXT,
    service TEXT,
    product_id TEXT REFERENCES erp_products(id),
    date TIMESTAMPTZ NOT NULL DEFAULT now(),
    status TEXT NOT NULL DEFAULT 'PENDING',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX website_bookings_tid_status ON website_bookings(template_id, status);
CREATE INDEX website_bookings_tid_date ON website_bookings(template_id, date DESC);

-- Seed Template 0 - Knowlia Dark
INSERT INTO website_templates (id, slug, name, version, is_active, theme_json) VALUES
('00000000-0000-0000-0000-000000000040', 'template-0', 'Knowlia Dark', '1.1.1', 1, '{"background":"#1a1a1a","foreground":"#fafafa","accent":"#3a6ea4","muted":"#bcbcbc"}');

INSERT INTO website_sections (id, template_id, kind, position, config_json, is_visible) VALUES
('00000000-0000-0000-0000-000000000041','00000000-0000-0000-0000-000000000040','hero',0,'{"title":"K N O W L I A","subtitle":"KNOWLEDGE + UTOPIA","badge":"Version 1.1.1","tagline":"Welcome to Knowlia: where knowledge meets innovation."}',1),
('00000000-0000-0000-0000-000000000042','00000000-0000-0000-0000-000000000040','slideshow',1,'{"slides":[{"url":"https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=1200","alt":"Preview 1"}]}',1),
('00000000-0000-0000-0000-000000000043','00000000-0000-0000-0000-000000000040','downloads',2,'{"platforms":[{"platform":"Windows","versions":[{"arch":"64-bit","url":"/app/builds/build-windows-x64/Knowlia_1.1.1_x64_en-US.msi","recommended":true}]}]}',1),
('00000000-0000-0000-0000-000000000044','00000000-0000-0000-0000-000000000040','products',3,'{"featured_product_ids":[]}',1);

INSERT INTO website_links (id, template_id, label, href, position, is_visible) VALUES
('00000000-0000-0000-0000-000000000045','00000000-0000-0000-0000-000000000040','Home','/',0,1),
('00000000-0000-0000-0000-000000000046','00000000-0000-0000-0000-000000000040','Products','/website/products',1,1),
('00000000-0000-0000-0000-000000000047','00000000-0000-0000-0000-000000000040','Cart','/website/cart',2,1),
('00000000-0000-0000-0000-000000000048','00000000-0000-0000-0000-000000000040','Bookings','/website/bookings',3,1);
