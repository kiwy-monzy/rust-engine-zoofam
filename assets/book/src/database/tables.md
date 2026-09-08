# The five tables

```
gateway_users ──┐
                ├──▶ gateway_user_roles ──▶ gateway_roles ──┐
                                                            ├──▶ gateway_role_permissions ──▶ gateway_permissions
```

Three tables hold things; two hold relationships between them. That pattern —
two entity tables and a join table between — is what "many-to-many" looks like in
SQL, and it appears twice here.

## gateway_users

```sql
CREATE TABLE gateway_users (
    id            VARCHAR(36)  PRIMARY KEY,
    email         VARCHAR(320) NOT NULL,
    email_lower   VARCHAR(320) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name  VARCHAR(120) NOT NULL DEFAULT '',
    avatar_url    VARCHAR(500) NOT NULL DEFAULT '',
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX gateway_users_email_lower_key ON gateway_users (email_lower);
```

Two things here are not obvious.

**`email_lower` exists so the unique index can be case-insensitive.** Without it,
`Me@example.com` and `me@example.com` are two different rows and one person has
two accounts. The original `email` is kept as typed, because that is how people
expect to see their own address. The unique index is on the lowercase copy.

There is a test for this: registering `DUPE@example.com` after `dupe@example.com`
answers `409 Conflict`.

**`is_active` is not the same as deleting.** A disabled account cannot log in but
its rows still exist, so audit trails and foreign keys stay intact. Sign-in
checks it explicitly and answers `403`, not `401` — see
[401 and 403](../routing/errors.md).

`VARCHAR(320)` is not arbitrary: 64 characters of local part, `@`, 255 of domain.

## gateway_roles and gateway_permissions

```sql
CREATE TABLE gateway_roles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        VARCHAR(64)  NOT NULL,
    description VARCHAR(255) NOT NULL DEFAULT ''
);

CREATE TABLE gateway_permissions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    module      VARCHAR(64)  NOT NULL,
    action      VARCHAR(64)  NOT NULL,
    description VARCHAR(255) NOT NULL DEFAULT ''
);
```

A permission is deliberately two columns rather than one string. `users` +
`read`, not `"users:read"`. It means you can ask "what may this role do to
users?" with a `WHERE module = 'users'` instead of a `LIKE`.

The `module:action` string you see in a token is built at the boundary, in
`controller::users::permission_keys`.

Roles are stored lowercase. `controller::roles::create` does that on the way in,
so `Admin` and `admin` cannot both exist.

## The two join tables

```sql
CREATE TABLE gateway_user_roles (
    user_id VARCHAR(36) NOT NULL REFERENCES gateway_users (id) ON DELETE CASCADE,
    role_id INTEGER     NOT NULL REFERENCES gateway_roles (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);
```

Three things are doing work in those four lines.

**The composite primary key** makes the same user-role pair impossible twice. Not
"unlikely" — impossible, enforced by the database, no matter what the application
does.

**`ON DELETE CASCADE`** means deleting a user removes their role assignments
automatically. Without it the delete fails, because rows still reference the user.

**The extra index on `role_id`**: the primary key indexes `(user_id, role_id)`,
which makes "what roles does this user have?" fast and "who has this role?" slow,
because the query cannot use an index that starts with the wrong column.

## Why user ids are strings

`VARCHAR(36)` holding a UUID like `16e6a7f7-c791-483f-b12d-1afb45e1b8e9`, rather
than a native `UUID` column or an auto-incrementing integer.

Not an integer, because sequential ids leak information — user 3 can see that
they were third, and can guess that user 4 exists. UUIDs are unguessable.

Not a native `UUID` column, because SQLite has no such type, and the whole point
of [two backends](backends.md) is that both share one `schema.rs`. On a
PostgreSQL-only project, use `UUID` with `gen_random_uuid()`.

The id is generated in Rust, in `controller::users::create`, not by the database.
