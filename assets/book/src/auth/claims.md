# What a token carries

```json
{
  "sub": "16e6a7f7-c791-483f-b12d-1afb45e1b8e9",
  "email": "me@example.com",
  "roles": ["admin"],
  "perms": ["permissions:read", "roles:read", "roles:write", "users:read", "users:write"],
  "jti": "a3f9c1e0-7b21-4e6a-9c3d-2f81b45e1b8e",
  "ver": 0,
  "iat": 1767225600,
  "exp": 1767254400
}
```

`sub` (subject) and `exp` (expiry) are standard JWT claim names. `roles` and
`perms` are ours.

## Resolved once, at login

The permission list is computed by walking three tables:

```
gateway_users → gateway_user_roles → gateway_role_permissions → gateway_permissions
```

```rust
let rows: Vec<(String, String)> = gateway_user_roles::table
    .inner_join(gateway_role_permissions::table.on(...))
    .inner_join(gateway_permissions::table.on(...))
    .filter(gateway_user_roles::user_id.eq(user_id))
    .select((gateway_permissions::module, gateway_permissions::action))
    .load(c)?;
```

The pairs become `module:action` strings, sorted and deduplicated — a user with
two roles that both grant `users:read` should carry it once.

Then it is **frozen into the token**. Authorizing a request touches no tables at
all, which is what makes the middleware cheap enough to run on every private
route.

## The trade

A token cannot notice that the world changed.

- Grant someone a role and their current token still lacks it.
- Revoke a role and they keep it until the token expires.
- Disable an account and they stay in until the token expires.

That last one is the one to think about. `is_active` is checked at login, not on
every request, so a disabled user with a fresh token has up to `JWT_TTL_HOURS`
of access left.

This is the fundamental trade of stateless auth: no database read per request, in
exchange for a window where the token disagrees with the database. `JWT_TTL_HOURS`
is that window. Eight hours suits a workday; if you need shorter, shorten it.

A test pins the behaviour so it is a decision and not an accident:

```rust
let (status, _) = call(&state, get("/users", Some(&token))).await;
assert_eq!(
    status,
    StatusCode::FORBIDDEN,
    "the token issued before the role was granted must not gain it"
);
```

## Closing the window

If you need instant revocation, the usual answers are:

- **Short expiry plus a refresh token.** Access tokens live minutes; a longer
  refresh token is checked against the database when exchanged.
- **A denylist.** Store revoked token ids and check on each request — which puts
  the database read back and costs you the reason you chose JWTs.
- **A version column.** Put `token_version` in the claims and in the user row;
  bump it to invalidate. One cheap indexed read per request.

This project implements the last two — a `jti` denylist for one-session logout
and a `token_version` for logout-everywhere. That deliberately trades the
stateless property away; see [Logging out for real](revocation.md) for how, and
what it costs.

## Asking the questions

```rust
impl Claims {
    pub fn has_role(&self, role: &str) -> bool { ... }

    pub fn can(&self, module: &str, action: &str) -> bool {
        let wanted = format!("{module}:{action}");
        self.perms.iter().any(|p| p == &wanted)
    }

    pub fn require(&self, module: &str, action: &str) -> Result<()> {
        if self.can(module, action) { Ok(()) } else {
            Err(AuthError::Forbidden(format!("{module}:{action}")))
        }
    }
}
```

Handlers call `require`, which produces an error naming the missing permission:

```json
{ "error": "this account may not users:read" }
```

Saying which permission was missing is a deliberate choice. It tells an
authenticated user what to ask an administrator for. It reveals only the name of
a permission, which the `/permissions` endpoint lists anyway.

**Check permissions, not roles.** `can("users", "read")` keeps working when you
add an `auditor` role; `has_role("admin")` does not. `has_role` is there for the
rare case where the role itself is the point.
