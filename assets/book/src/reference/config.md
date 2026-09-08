# Configuration reference

Read from the environment, or from a `.env` file in the working directory via
`dotenvy`.

| Variable | Default | Meaning |
|---|---|---|
| `DATABASE_URL` | `admin.db` (SQLite) | SQLite file path or PostgreSQL connection URL |
| `JWT_SECRET` | `dev-secret-change-me` | HS256 signing key |
| `JWT_TTL_HOURS` | `8` | how long an issued token stays valid |
| `PORT` | `8080` | listen port on `127.0.0.1` |
| `RUST_LOG` | `gateway=info,routes=info,tower_http=info` | log filter |

## DATABASE_URL

The default depends on which backend was compiled in, so its meaning changes with
the cargo feature. Under `postgres` it becomes
`postgres://postgres:postgres@localhost/admin_app`.

SQLite accepts a path (`admin.db`, `/var/lib/app/admin.db`) or the special
`:memory:`, which the tests use — read
[Two bugs worth knowing](../practice/pitfalls.md) before using that anywhere
else.

PostgreSQL takes a standard URL: `postgres://user:password@host:port/database`.

## JWT_SECRET

The most sensitive value here. Anyone holding it can mint a token for any user
with any permission, no password required.

- Change it before anything real depends on it.
- Keep it out of git; `.gitignore` covers `.env`.
- Use something long and random: `openssl rand -base64 48`.
- Changing it invalidates every existing token at once. That is the emergency
  lever if one leaks.

## JWT_TTL_HOURS

How long a token remains valid, and therefore **how long a revoked permission
keeps working**. See [What a token carries](../auth/claims.md).

Eight hours suits a workday: sign in once, no interruptions. Shorten it if prompt
revocation matters more than convenience.

## PORT

Binds `127.0.0.1` only, so it accepts local connections. To listen more widely,
change the address in `gateway/src/main.rs`:

```rust
let addr = SocketAddr::from(([127, 0, 0, 1], port));
```

Think before you do. There is no TLS here and no rate limiting; tokens and
passwords would cross the network in the clear. Put it behind a reverse proxy
that terminates TLS.

## RUST_LOG

Standard `tracing` filter syntax:

```bash
RUST_LOG=debug cargo run -p gateway
RUST_LOG=gateway=info,tower_http=debug cargo run -p gateway
```

`tower_http=debug` logs every request and response, which is the setting to reach
for when a route is not behaving.

## Example .env

```
DATABASE_URL=admin.db
JWT_SECRET=dev-secret-change-me
JWT_TTL_HOURS=8
PORT=8080
```

Checked in as an example only, with an obviously fake secret. A real deployment
sets these in the environment.
