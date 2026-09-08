# Diesel in practice

Diesel is a query builder that checks your SQL at **compile time**. Misspell a
column and the program does not build. The cost is a type system you have to
meet halfway.

## The three pieces

**`schema.rs`** describes the tables:

```rust
diesel::table! {
    gateway_roles (id) {
        id -> Int4,
        #[max_length = 64]
        name -> Varchar,
    }
}
```

This macro generates a module of types. `gateway_roles::name` is not a string —
it is a value with a type that knows it belongs to that table and holds text.
That is what makes the compile-time checking possible.

**Model structs** map rows to Rust:

```rust
#[derive(Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_roles)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: String,
}
```

- `Queryable` — can be built from a result row
- `Selectable` — can generate its own `SELECT` list via `Role::as_select()`
- `Identifiable` — has a primary key, needed for joins
- `Insertable` — can be written (on the `New*` structs)
- `AsChangeset` — can be used in `UPDATE` (on `UserPatch`)

**Queries** compose from those:

```rust
gateway_users::table
    .filter(gateway_users::email_lower.eq(email))
    .select(User::as_select())
    .first(&mut c)
```

## Why `as_select()` and not `SELECT *`

`Queryable` alone matches columns **by position**. Add a column to the middle of a
table and every struct silently shifts by one — a `String` lands where another
`String` was expected and the compiler is happy while your data is wrong.

`Selectable` plus `as_select()` matches **by name** and generates the column list
from the struct. Add a column anywhere and nothing breaks.

## Two ids in a join

```rust
gateway_user_roles::table
    .inner_join(
        gateway_role_permissions::table
            .on(gateway_role_permissions::role_id.eq(gateway_user_roles::role_id)),
    )
```

Diesel can infer the `ON` clause from `joinable!` declarations in `schema.rs`,
but only for one path between two tables. This query joins three tables through
two different columns, so the condition is spelled out.

## `allow_tables_to_appear_in_same_query!`

Diesel will not let two tables appear in one query unless you have said they may.
It is the mechanism that stops you accidentally writing a cross join between
unrelated tables.

## Errors worth translating

```rust
diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)
    => Error::Conflict("that email is already registered".into())
```

`UniqueViolation` is the database telling you a constraint stopped an insert. It
becomes a `409` with a sentence a person can act on, rather than a `500` and a
line in a log.

`Error::NotFound` is Diesel's way of saying `first()` matched nothing. It becomes
a `404`.

Everything else becomes a `500` and the detail goes to the log, not the response —
database errors can contain table and column names, and those are not the
caller's business.

## Connections come from a pool

Opening a database connection is expensive, so `r2d2` keeps a few open and hands
them out. `db::conn(&pool)` borrows one; dropping it returns it.

Two rules follow, and both are the subject of
[Two bugs worth knowing](../practice/pitfalls.md):

1. One request should check out one connection, not several.
2. Never ask for a second connection while holding the first.
