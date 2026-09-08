# Two backends, one switch

```bash
cargo run -p gateway                                          # SQLite
cargo run -p gateway --no-default-features --features postgres
```

One codebase, two databases, chosen at compile time by a cargo feature.

## What the feature selects

`db/src/lib.rs` picks four things:

```rust
#[cfg(feature = "postgres")]
pub type Connection = diesel::pg::PgConnection;
#[cfg(feature = "sqlite")]
pub type Connection = diesel::sqlite::SqliteConnection;

#[cfg(feature = "postgres")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations/postgres");
#[cfg(feature = "sqlite")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations/sqlite");
```

Everything downstream is written against `db::Connection` and never names a
concrete database type, so `controller` compiles unchanged either way.

## Refusing the nonsense combinations

```rust
#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("pick one backend: --features postgres or --features sqlite, not both");

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("pick one backend: --features postgres or --features sqlite");
```

Cargo features are additive — turning one on never turns another off — so
nothing stops someone enabling both. Without these, `Connection` would be
declared twice and you would get a confusing type error deep in Diesel. This
gives you the sentence you actually needed.

## Forwarding the choice

Every crate above `db` carries the same pair of features and passes them down:

```toml
[features]
default = ["sqlite"]
postgres = ["db/postgres", "models/postgres", "controller/postgres"]
sqlite   = ["db/sqlite",   "models/sqlite",   "controller/sqlite"]
```

There is a trap here worth knowing. If a workspace dependency does not say
`default-features = false`, a member crate saying it is **silently ignored** —
cargo warns and carries on, and you get SQLite compiled into your Postgres build.
So the workspace declares:

```toml
db = { path = "db", default-features = false }
```

## How different is the SQL really?

Two lines. That is the entire difference between the two migration sets:

```sql
id  SERIAL PRIMARY KEY                    -- postgres
id  INTEGER PRIMARY KEY AUTOINCREMENT     -- sqlite
```

Everything else — `VARCHAR`, `BOOLEAN`, `TIMESTAMP`, `REFERENCES`,
`ON DELETE CASCADE`, composite primary keys, `CREATE UNIQUE INDEX` — is spelled
identically in both.

That is only true because of choices made elsewhere: string ids instead of native
`UUID`, and `CURRENT_TIMESTAMP` instead of `NOW()`. Get those right and supporting
two backends costs almost nothing.

## Foreign keys are off by default in SQLite

```rust
c.batch_execute("PRAGMA foreign_keys = ON;")?;
```

SQLite ignores `REFERENCES` unless you ask it not to, per connection. Without
this line every `ON DELETE CASCADE` in the schema is decoration, and deleting a
user silently leaves orphaned rows in `gateway_user_roles`.

PostgreSQL enforces them always, which is why the Postgres version of that
function does nothing.

## The build requirement nobody mentions

**SQLite needs nothing.** `libsqlite3-sys` with the `bundled` feature compiles the
entire engine from C source into your binary.

**PostgreSQL needs `libpq` installed.** Diesel links against the real PostgreSQL
client library, and if it is missing the build fails in `pq-sys` before your code
is even considered:

```
Folder "PQ_LIB_DIR" doesn't exist in the configured path
```

Install PostgreSQL, or point `PQ_LIB_DIR` at an existing `lib` directory.

> The PostgreSQL build in this project has **never been compiled**, because
> PostgreSQL is not installed on the machine it was written on. The SQL and the
> feature wiring are written and reviewed, but untested. Treat it as a starting
> point, not a guarantee.
