# Logging out for real

A stateless JWT cannot be un-issued. Once signed, it is valid until it expires,
and nothing the server does — including "log out" — changes that. The only way
to make logout actually revoke a token is to give the server something to check
on every request. That is what this does, and it is worth being clear that it
undoes the very property that made the token stateless.

## Two kinds of logout

- **`POST /auth/logout`** — ends *this* session. The token you called it with
  stops working; your other sessions do not.
- **`POST /auth/logout-all`** — ends *every* session for the account at once.
  The phone, the laptop, the forgotten tab in a library — all of them.

Both are private routes: you prove who you are with the token you are revoking.

## How each is built

Two mechanisms, because the two questions are different.

**One token, by id.** Every token now carries a `jti` — a random id, unique per
login (see [What a token carries](claims.md)). `logout` writes that id into a
`gateway_revoked_tokens` table:

```rust
pub fn revoke(pool: &DbPool, claims: &Claims) -> Result<()> {
    let record = NewRevokedToken {
        jti: claims.jti.clone(),
        user_id: claims.sub.clone(),
        expires_at: /* the token's own exp */,
    };
    diesel::insert_into(gateway_revoked_tokens::table)
        .values(&record)
        .on_conflict_do_nothing()
        .execute(&mut c)?;
    purge_expired(&mut c)
}
```

The row is stored with the token's own expiry, and expired rows are purged on
each insert — the denylist never grows past the tokens that could still be live.

**Every token, by version.** The `gateway_users` row has a `token_version`, and
each token records the version it was minted at (`ver`). `logout-all` bumps the
column:

```rust
pub fn revoke_all(pool: &DbPool, user_id: &str) -> Result<()> {
    diesel::update(gateway_users::table.find(user_id))
        .set(gateway_users::token_version.eq(gateway_users::token_version + 1))
        .execute(&mut c)
}
```

Every token issued before the bump now carries the old version, and fails the
check below. No per-token bookkeeping, one integer.

## The check that makes it real

The middleware used to stop at "the signature is good". Now it asks the database
whether the session is still alive:

```rust
pub fn validate(pool: &DbPool, claims: &Claims) -> Result<()> {
    let (is_active, token_version): (bool, i32) = gateway_users::table
        .find(&claims.sub)
        .select((gateway_users::is_active, gateway_users::token_version))
        .first(&mut c)
        .map_err(|_| Error::SessionEnded("the account no longer exists"))?;

    if !is_active {
        return Err(Error::SessionEnded("the account is disabled"));
    }
    if token_version != claims.ver {
        return Err(Error::SessionEnded("signed out everywhere; sign in again"));
    }

    let revoked = /* does claims.jti exist in gateway_revoked_tokens? */;
    if revoked {
        return Err(Error::SessionEnded("this session was signed out"));
    }
    Ok(())
}
```

Three failures, all mapped to **401** — the client should log in again. And it
buys a fourth thing for free: `is_active` is now enforced on *every* request, so
disabling an account ends its sessions immediately. That was an
[exercise](../practice/exercises.md) in the earlier version; here it falls out of
the same read.

## The cost, stated plainly

This is one database read per authenticated request. The whole point of a
stateless JWT — covered in [What a token carries](claims.md) — was to avoid
exactly that. Revocation and statelessness are in direct tension, and you cannot
have both: a token the server never checks is a token the server cannot recall.

What keeps the cost small:

- It is a single indexed read on a primary key (`users.id`), plus an existence
  check on another primary key (`revoked_tokens.jti`).
- The denylist is bounded — it only ever holds unexpired revoked tokens, and
  purges itself.
- It replaces nothing that was cheaper; the middleware already did work.

If even that read is too much for your traffic, the usual escape is to keep
access tokens *very* short (minutes) so revocation is rarely needed, and pair
them with a longer refresh token that is checked only when exchanged. That moves
the read from every request to every few minutes. It is more moving parts; this
project chose the simpler thing that is easy to reason about.

## What it proves in tests

```rust
fn logout_revokes_the_token_it_was_called_with()
fn logout_revokes_only_that_session_not_the_others()
fn logout_all_revokes_every_existing_session()
fn a_disabled_account_is_rejected_on_the_next_request()
```

The second is the interesting one: it logs in twice, revokes one token, and
asserts the *other* still works. Per-session logout that quietly killed every
session would pass a naive test and annoy every user with two devices.
