# app — a small RBAC gateway, built to be read

A working HTTP API with users, roles, permissions and JWT auth. It is deliberately
small and split into six crates so you can see where each responsibility lives.

The code has no comments. This file is the explanation.

## The book

This README is the short version. The long one is an mdBook in `book/`:

```bash
cd book
mdbook serve --open
```

Eighteen chapters covering Diesel migrations, the two-backend switch, password
hashing, JWTs, the public/private route split, and the two bugs that were hit
while building it. Install it with `cargo install mdbook` if you do not have it.

---

## Run it

```bash
cd app
cargo run -p gateway
```

That is the whole setup. It uses SQLite by default, creates `admin.db` in the
current directory, applies all seven migrations on startup, and listens on
`http://127.0.0.1:8080`.

```bash
cargo test --workspace
```

20 tests, no database server needed.

---

## The six crates, and why there are six

```
gateway  ──▶ routes ──▶ controller ──▶ models ──▶ db
                 └────▶ auth
```

Each arrow is a dependency. Nothing points backwards, and that is the point:
you can read any crate knowing it cannot secretly reach into the ones above it.

| Crate | Answers one question | Knows about |
|---|---|---|
| `db` | how do I get a connection, and are the migrations applied? | Diesel, the pool, nothing else |
| `models` | what shapes live in the database? | the tables and the Rust structs for them |
| `controller` | what can be *done* with those shapes? | queries, validation, hashing on the way in |
| `auth` | who is this, and may they? | passwords and JWTs, no database at all |
| `routes` | which URL runs which of those? | HTTP, the public/private split, error codes |
| `gateway` | wire it together and listen | almost nothing — 40 lines |

`auth` not depending on `db` is the interesting one. It never loads a user. It
hashes passwords, signs tokens and reads them back. The *database* half of
signing in lives in `controller::users`, and `routes` is where the two meet.
That is why `auth` has 7 unit tests that need no database.

---

## Migrations: one table each

```
migrations/
  sqlite/                         postgres/
    ...000001_create_gateway_users/       (the same five)
    ...000002_create_gateway_roles/
    ...000003_create_gateway_permissions/
    ...000004_create_gateway_role_permissions/
    ...000005_create_gateway_user_roles/
```

Every folder has `up.sql` and `down.sql`. `up` creates one table; `down` drops it.
Diesel runs them in filename order, which is why they are numbered — table 4
references tables 2 and 3, so those must exist first.

Diesel records which ones ran in a `__diesel_schema_migrations` table it manages
itself. Running the gateway twice does not re-apply them.

Migrations 2, 3 and 4 also `INSERT`. That is deliberate: an `admin` role with
every permission and a `viewer` role with the read-only ones exist the moment the
database does, so there is something to log in as without a seed script.

### Adding a table

```bash
cargo install diesel_cli --no-default-features --features sqlite
diesel migration generate create_gateway_sessions --migration-dir migrations/sqlite
```

Write the `up.sql` and `down.sql`, add the same folder under `migrations/postgres`,
then add a `diesel::table!` block to `models/src/schema.rs`. The schema file is
checked in rather than generated, because it has to describe both backends.

### Undoing one

```bash
diesel migration revert --migration-dir migrations/sqlite
```

---

## Two backends, one switch

```bash
cargo run -p gateway                                          # SQLite
cargo run -p gateway --no-default-features --features postgres
```

`db` picks the connection type, the migration folder and the default URL from
the feature, and refuses to build if you ask for both or neither. Every crate
above it forwards the choice, which is why they all carry the same pair of
features.

The two SQL sets differ in exactly **two lines**:

```sql
id  SERIAL PRIMARY KEY                    -- postgres
id  INTEGER PRIMARY KEY AUTOINCREMENT     -- sqlite
```

That is only true because user ids are `VARCHAR(36)` holding a UUID string
rather than a native `UUID` column. A real Postgres-only schema would use `UUID`
with `gen_random_uuid()`; sharing one `schema.rs` and one set of model structs
across both backends is worth more here than the native type.

**PostgreSQL needs `libpq` installed to compile** — Diesel links against the real
client library. On this machine it is not installed, and `PQ_LIB_DIR` points at
`C:\Program Files\PostgreSQL\17\lib`, which does not exist. So the Postgres
build is written but has never been compiled or run here. Install PostgreSQL, or
clear that variable, and it should work. The SQLite build needs nothing: it
compiles the engine into the binary.

---

## Public and private routes

This is the part worth understanding, and it is nine lines in `routes/src/lib.rs`:

```rust
pub fn public(state: AppState) -> Router {
    Router::new()
        .route("/health", get(controllers::health))
        .route("/auth/register", post(controllers::register))
        .route("/auth/login", post(controllers::login))
        .with_state(state)
}

pub fn private(state: AppState) -> Router {
    Router::new()
        .route("/users", get(controllers::list_users) /* ... */)
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::require_auth))
        .with_state(state)
}
```

Two routers. The private one has `require_auth` attached with `route_layer`, and
the public one does not. `app()` merges them.

**`route_layer` is the load-bearing word.** With `.layer()` the middleware runs
for every request that reaches the router, including ones that match no route —
so an unknown URL would answer 401 instead of 404, telling an attacker that a
path exists whenever it *doesn't* 401. `route_layer` runs only after a route has
matched.

A route is private because of which function it was registered in. There is no
list of exceptions to keep in sync, and no way to add a route and forget to
protect it — if you put it in `private()`, it is protected.

### The full table

| Method | Path | Auth | Permission |
|---|---|---|---|
| GET | `/health` | public | — |
| POST | `/auth/register` | public | — |
| POST | `/auth/login` | public | — |
| GET | `/auth/me` | private | any valid token |
| POST | `/auth/logout` | private | any valid token |
| POST | `/auth/logout-all` | private | any valid token |
| GET | `/users` | private | `users:read` |
| POST | `/users` | private | `users:write` |
| GET | `/users/{id}` | private | `users:read` |
| PATCH | `/users/{id}` | private | `users:write` |
| DELETE | `/users/{id}` | private | `users:write` |
| POST/DELETE | `/users/{id}/roles/{role_id}` | private | `users:write` |
| GET | `/roles` | private | `roles:read` |
| POST | `/roles` | private | `roles:write` |
| GET | `/roles/{id}` | private | `roles:read` |
| DELETE | `/roles/{id}` | private | `roles:write` |
| POST/DELETE | `/roles/{id}/permissions/{permission_id}` | private | `roles:write` |
| GET | `/permissions` | private | `permissions:read` |
| POST | `/permissions` | private | `roles:write` |
| DELETE | `/permissions/{id}` | private | `roles:write` |

`/auth/register` being public is a choice, not a rule. It is what lets you create
the first account on an empty database. Close it by moving that one line from
`public()` to `private()`; `POST /users` already does the same job for signed-in
admins.

---

## Two different "no"s

They are not the same and the API must not confuse them:

- **401 Unauthorized** — *I do not know who you are.* No token, a forged one, an
  expired one. From `require_auth`, before the handler runs.
- **403 Forbidden** — *I know who you are, and no.* A valid token whose claims
  lack the permission. From `claims.require("users", "read")` inside the handler.

Which is why permission checks are in the handlers rather than the middleware:
the middleware cannot know that `GET /users` wants `users:read` while
`POST /users` wants `users:write`.

---

## What a token carries

```json
{
  "sub": "0b7c…", "email": "a@b.c",
  "roles": ["admin"],
  "perms": ["roles:read", "roles:write", "users:read", "users:write"],
  "iat": 1767225600, "exp": 1767254400
}
```

Permissions are resolved at login — user → roles → role_permissions →
permissions — and **frozen into the token**. Nothing hits the database to
authorize a request, which is what makes `require_auth` cheap.

The cost is that a token does not notice changes. Grant someone a role and their
current token still lacks it until they log in again; revoke one and they keep it
until the token expires. `JWT_TTL_HOURS` (default 8) is how long you are willing
to wait. There is a test for exactly this, so the behaviour is pinned rather than
accidental.

---

## Passwords

`auth::hash_password` uses Argon2id with a random salt per password, so the same
password hashes differently every time — there is a test asserting that. The hash
is never returned by the API: `User.password_hash` and `email_lower` are
`#[serde(skip)]`, and a test greps the `/auth/me` response to make sure.

Wrong password and unknown email both answer *"the email or password is wrong"*.
Distinguishing them tells an attacker which addresses are registered.

---

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `DATABASE_URL` | `admin.db` (SQLite) | SQLite file path, or a Postgres URL |
| `JWT_SECRET` | `dev-secret-change-me` | HS256 signing key — **change this** |
| `JWT_TTL_HOURS` | `8` | how long a token lasts |
| `PORT` | `8080` | listen port |
| `RUST_LOG` | `gateway=info,routes=info` | log filter |

Read from `.env` via `dotenvy`. Anyone holding `JWT_SECRET` can mint a token for
any user, so it does not belong in git.

---

## Try it

```bash
curl -s localhost:8080/health

curl -s -X POST localhost:8080/auth/register \
  -H 'content-type: application/json' \
  -d '{"email":"me@example.com","password":"password123","display_name":"Me"}'

TOKEN=$(curl -s -X POST localhost:8080/auth/login \
  -H 'content-type: application/json' \
  -d '{"email":"me@example.com","password":"password123"}' | jq -r .token)

curl -s localhost:8080/users
curl -s localhost:8080/users -H "authorization: Bearer $TOKEN"
```

The last two are the lesson. Without the header: 401. With it: 403, because a
fresh account has no role. Give yourself one, log in again, and it becomes 200:

```bash
sqlite3 admin.db "INSERT INTO gateway_user_roles (user_id, role_id)
  SELECT u.id, r.id FROM gateway_users u, gateway_roles r
  WHERE u.email_lower='me@example.com' AND r.name='admin';"
```

---

## Two bugs worth knowing about

Both were hit while building this, and both are easy to hit again.

**In-memory SQLite gives each connection its own database.** The tests use
`DATABASE_URL=:memory:`. Migrations ran on one pooled connection and the queries
ran on another — which was empty. `db::pool_size` caps in-memory pools at one
connection. A file-backed or Postgres database does not have this problem.

**Nested connection checkouts deadlock a one-connection pool.** `users::list`
held a connection and called `role_names`, which asked the pool for a second.
With eight connections that silently worked; with one it would hang forever. The
functions that need a connection now take `&mut DbConn` and the public ones
check out exactly once. This is a good habit at any pool size — one request
should hold one connection.
