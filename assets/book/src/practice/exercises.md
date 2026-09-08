# Exercises

Roughly in order of difficulty. Each is a real change to this codebase.

## 1. Close registration

Move `/auth/register` from `public()` to `private()` in `routes/src/lib.rs`.

Now `POST /users` is the only way to create an account, and it needs
`users:write`. Which test breaks, and why? What would you have to do to create
the first user on an empty database?

## 2. Add a permission and use it

Add a migration creating `permissions:write`, grant it to `admin`, and change
`create_permission` and `delete_permission` to require it instead of
`roles:write`.

Two migrations to write, one per backend. Do the already-seeded roles pick it up
automatically, or does migration 4 need to run again? Read
[Migrations](../database/migrations.md) before answering.

## 3. Make `is_active` mean something on every request

Today a disabled account keeps working until its token expires. Make the
middleware reject a disabled user immediately.

You will need a database read in `require_auth`. Write down what that costs
before you write the code: which page in this book warned you, and what did it
suggest instead?

## 4. Add a sixth table

`gateway_audit_log`: who did what, when, and whether it worked.

```
id, ts, user_id, email, action, target, detail, ok
```

One migration per backend, a `diesel::table!` block, a model struct, and calls
from the handlers that change something. Note that `user_id` should be nullable —
a failed login has no user.

## 5. Refresh tokens

Make access tokens last fifteen minutes and add a refresh token lasting a week,
stored in a new table so it can be revoked.

New endpoint: `POST /auth/refresh`. What happens to a refresh token when it is
used — is it reusable, or replaced? Both answers are defensible; know why you
picked yours.

## 6. Pagination

`GET /users` returns every user. At ten thousand accounts that is a problem.

Add `?limit=` and `?offset=`, cap `limit` at something sane, and return a total
count. Then read about why offset pagination gets slow on large tables, and what
cursor pagination does instead.

## 7. Make the tests hostile

The suite covers the happy paths and the obvious failures. Add tests for:

- a token with a valid signature but a `sub` that is not a real user
- `PATCH /users/{id}` with an empty body
- a role name 500 characters long
- two simultaneous registrations of the same email

The last is the interesting one. What stops both from succeeding — your code, or
the database?

## 8. Compare it with the real thing

This project is a sibling of `crates/api-gateway`, which does the same job with
`hyper` instead of `axum`, hand-written SQL instead of Diesel, and a great deal
more besides.

Read its `controllers/auth.rs` next to `routes/src/controllers.rs`. The concepts
match; the machinery does not. Which decisions were made differently, and can you
work out why?
