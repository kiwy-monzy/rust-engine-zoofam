# Admin Website Architecture

A Rust web admin dashboard built with Tauri, featuring JWT authentication, RBAC permissions, and multiple business modules.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust (Axum framework) |
| Database | SQLite / PostgreSQL (Diesel ORM) |
| Desktop | Tauri |
| Frontend | React + TypeScript + Vite |
| Auth | JWT + Argon2 password hashing |
| Search | Tantivy full-text search |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Client Layer                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Frontend │  │ Mobile   │  │  Apple   │  │   Map Clients    │ │
│  │ React/TS│  │   Apps   │  │  Wallet  │  │   (Tiles)        │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────────┬─────────┘ │
└───────┼─────────────┼─────────────┼─────────────────┼───────────┘
        │             │             │                 │
        └─────────────┴──────┬──────┴─────────────────┘
                             │ HTTP
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Gateway (Port 8080)                        │
│                    gateway crate - 40 lines                     │
└─────────────────────────────┬───────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
          ▼                   ▼                   ▼
    ┌──────────┐       ┌──────────┐       ┌────────────┐
    │  Public  │       │  Private │       │Wallet Public│
    │  Routes  │       │  Routes  │       │  Routes    │
    └────┬─────┘       └────┬─────┘       └─────┬──────┘
         │                   │                   │
         │    ┌──────────────┼──────────────┐    │
         │    │              │              │    │
         ▼    ▼              ▼              ▼    ▼
    ┌─────────────────────────────────────────────────┐
    │                   Routes Layer                    │
    │  ┌──────┐ ┌─────┐ ┌──────┐ ┌─────────┐        │
    │  │ Auth │ │ ERP │ │ CRM  │ │  DMC    │  ...   │
    │  └──────┘ └─────┘ └──────┘ └─────────┘        │
    └────────────────────┬──────────────────────────┘
                         │
                         ▼
    ┌─────────────────────────────────────────────────┐
    │                Controller Layer                  │
    │            Business logic & validation          │
    └────────────────────┬──────────────────────────┘
                         │
                         ▼
    ┌─────────────────────────────────────────────────┐
    │                 Models Layer                     │
    │              Rust structs & queries              │
    └────────────────────┬──────────────────────────┘
                         │
                         ▼
    ┌─────────────────────────────────────────────────┐
    │                    db Layer                      │
    │        Diesel ORM + connection pool             │
    └────────────────────┬──────────────────────────┘
                         │
                         ▼
                   ┌──────────┐
                   │ Database │
                   │SQLite/PG │
                   └──────────┘
```

## Request Flow

1. **Client** sends HTTP request to `gateway:8080`
2. **Gateway** routes to public, private, or wallet router
3. **Public routes** skip auth: `/health`, `/auth/login`, `/auth/register`, tiles, public storage
4. **Private routes** require JWT via `require_auth` middleware
5. **Routes module** dispatches to appropriate handler
6. **Controller** validates input, applies business logic
7. **Models** transform data to/from database structs
8. **db** executes queries via Diesel connection pool

## Module Overview

### Core Crates (Dependency Chain)

```
gateway ──► routes ──► controller ──► models ──► db
               └────► auth
```

| Crate | Responsibility | Dependencies |
|-------|----------------|---------------|
| `db` | Connection pool, migrations | Diesel, r2d2 |
| `models` | Database structs & schema | db |
| `controller` | Business logic, validation | models, auth |
| `auth` | JWT signing, password hashing | none (stateless) |
| `routes` | HTTP routing, middleware | controller, auth |
| `gateway` | App assembly, server start | routes |

### Business Modules

#### ERP (Enterprise Resource Planning)
7 sub-modules for ledger-based business operations.

#### CRM (Customer Relationship Management)
Quote → Sales Order → Accounting Invoice workflow.

#### DMC (Destination Management Company)
Tour operator module for managing destinations, packages, and bookings.

#### Marketplace (Fundi Service)
Service marketplace connecting providers with customers.

#### Website (CMS)
Template-based website builder with:
- Hero sections
- Slideshows
- Downloads
- Links
- Products
- Cart
- Bookings

#### Fleet Management
- Vehicle tracking
- Map events
- Real-time location updates

#### Maps
- Tile server for map clients
- Layer management
- GeoJSON import

#### Search
Full-text search powered by Tantivy with Jieba tokenization.

#### Storage
User file storage with avatar support and uploads (25MB limit).

#### Wallet
Apple Pass integration:
- Pass download
- Device registration
- Push updates

## API Routes

### Public Routes (No Auth Required)
```
GET  /health
POST /auth/register
POST /auth/login
POST /auth/refresh
POST /auth/password-reset/request
POST /auth/password-reset/confirm
GET  /tiles/{z}/{x}/{y}
GET  /storage/public/{path}
GET  /wallet/pass/{pass_type}/{serial_pkpass}
POST /wallet/v1/devices/{device_id}/registrations/{pass_type}/{serial}
GET  /wallet/v1/devices/{device_id}/registrations/{pass_type}
GET  /wallet/v1/passes/{pass_type}/{serial}
POST /wallet/v1/log
```

### Private Routes (JWT Required)
```
# Auth
GET  /auth/me
POST /auth/logout
POST /auth/logout-all
GET  /auth/profile
PATCH /auth/profile
POST /auth/profile/avatar
POST /auth/profile/avatar/pick
GET  /auth/profile/avatars
POST /auth/profile/password
GET  /auth/profile/sessions
DELETE /auth/profile/sessions/:id

# Users & RBAC
GET    /users
POST   /users
GET    /users/search
GET    /users/:id
PATCH  /users/:id
DELETE /users/:id
POST   /users/:id/roles/:role_id
DELETE /users/:id/roles/:role_id
GET    /roles
POST   /roles
GET    /roles/:id
PATCH  /roles/:id
DELETE /roles/:id
POST   /roles/:id/permissions/:permission_id
DELETE /roles/:id/permissions/:permission_id
GET    /permissions
POST   /permissions
PATCH  /permissions/:id
DELETE /permissions/:id

# System
GET  /system
PATCH /system
GET  /system/version
GET  /system/releases
POST /system/releases
GET  /system/releases/latest
GET  /system/releases/check/:version
GET  /system/releases/:id/download
DELETE /system/releases/:id

# Support
GET  /support
POST /support
GET  /support/users
POST /support/:id/close

# Business Modules
* /erp/*       - ERP operations
* /crm/*       - CRM operations
* /dmc/*       - DMC operations
* /marketplace/* - Marketplace operations
* /website/*   - Website CMS

# Fleet & Maps
* /fleet/*     - Vehicle management
* /maps/*      - Map layers & tiles
* /uploads/*   - GeoJSON import (25MB limit)

# Search
GET  /search
POST /search/reindex

# Storage
* /storage/*   - User file storage
```

## Authentication Flow

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  User   │────►│ /login  │────►│ JWT     │────►│ Request │
│         │◄────│         │◄────│ Token   │     │ w/ JWT  │
└─────────┘     └─────────┘     └─────────┘     └─────────┘
                                                 │
                                                 ▼
                                          ┌─────────────┐
                                          │ Middleware  │
                                          │ validate_jwt│
                                          └─────────────┘
```

### JWT Token Structure
```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "roles": ["admin"],
  "perms": ["users:read", "users:write", ...],
  "iat": 1767225600,
  "exp": 1767254400
}
```

Permissions are resolved at login (user → roles → permissions) and frozen into the token.

## Database Schema

### Core Tables
- `gateway_users` - User accounts
- `gateway_roles` - RBAC roles
- `gateway_permissions` - Permission definitions
- `gateway_role_permissions` - Role-permission mapping
- `gateway_user_roles` - User-role mapping

### Business Tables
- ERP tables (ledgers, accounts, transactions)
- CRM tables (quotes, orders, invoices)
- DMC tables (destinations, packages, bookings)
- Marketplace tables (services, providers, orders)
- Website tables (pages, products, cart, bookings)
- Fleet tables (vehicles, locations, events)
- Map tables (layers, tiles, GeoJSON data)

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `admin.db` | SQLite file or Postgres URL |
| `JWT_SECRET` | `dev-secret` | HS256 signing key |
| `JWT_TTL_HOURS` | `8` | Token expiration |
| `PORT` | `8080` | Server port |
| `RUST_LOG` | `gateway=info` | Log filter |

## Running

```bash
# Start gateway (SQLite default)
cargo run -p gateway

# Start with PostgreSQL
cargo run -p gateway --no-default-features --features postgres

# Run tests
cargo test --workspace

# Serve frontend dev server
cd frontend && npm run dev
```
