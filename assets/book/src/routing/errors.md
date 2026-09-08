# 401 and 403 are different

They are two different sentences and the API must not confuse them.

| | Means | Decided by |
|---|---|---|
| **401 Unauthorized** | *I do not know who you are.* | `require_auth`, before the handler |
| **403 Forbidden** | *I know who you are, and no.* | `claims.require(...)`, inside the handler |

The names are unhelpful — "Unauthorized" is the one about *authentication*. Read
401 as "unauthenticated" and it stops being confusing.

## Seeing both

```bash
curl -s -o /dev/null -w '%{http_code}\n' localhost:8080/users
# 401 — no token at all

curl -s localhost:8080/users -H "authorization: Bearer $TOKEN"
# 403 — {"error":"this account may not users:read"}
```

The second is a real, valid, correctly signed token belonging to a real account.
It just has no roles, because a freshly registered user has none.

## Why the difference matters

A client can act on it:

- **401** — the token is missing, stale or wrong. Send the user to log in.
- **403** — logging in again changes nothing. The user needs a role.

An API that answers 401 for both sends users into a login loop that cannot
possibly help. One that answers 403 for both leaves an expired session looking
like a permissions problem, and the user contacts an administrator who finds
nothing wrong.

## Where each is decided

**401** comes from the middleware, before any handler runs:

```rust
let token = auth::bearer(header)?;   // Missing  -> 401
let claims = state.jwt.verify(token)?; // Invalid/Expired -> 401
```

**403** comes from the handler:

```rust
claims.require("users", "read")?;   // Forbidden -> 403
```

Which raises the obvious question.

## Why permissions are not checked in the middleware

Because the middleware does not know what the route needs.

`GET /users` needs `users:read`. `POST /users` needs `users:write`. Same path,
different requirement. The middleware sees a path and a method; it does not know
what they mean.

Two ways to fix that, both worse:

- **A table of path patterns to permissions.** Now the requirement lives away
  from the handler, and a renamed route silently loses its check.
- **A separate middleware layer per permission.** Workable, but the wiring grows
  faster than the handlers do.

Putting the check on the first line of the handler keeps it next to the code it
protects. It is visible in review, and a handler with no `require` call is
visibly missing one.

## The full mapping

```rust
impl From<AuthError> for ApiError {
    fn from(e: AuthError) -> Self {
        let status = match e {
            AuthError::Missing | AuthError::Invalid(_) | AuthError::Expired => UNAUTHORIZED,
            AuthError::BadCredentials => UNAUTHORIZED,
            AuthError::Disabled | AuthError::Forbidden(_) => FORBIDDEN,
            AuthError::Hashing(_) => INTERNAL_SERVER_ERROR,
        };
        ApiError::new(status, e.to_string())
    }
}
```

`Disabled` is 403, not 401. The credentials were right — the account is switched
off. Answering 401 would invite the user to try their password again, which will
keep working and keep failing.

`Hashing` is a 500 because it means the hasher itself failed. That is the
server's problem, not the caller's.

## From the controller

```rust
E::NotFound(_)  => 404
E::Conflict(_)  => 409   // that email is already registered
E::Invalid(_)   => 400   // the password must be at least 8 characters
E::Db(inner)    => 500   // logged, not returned
```

409 for a duplicate rather than 400: the request was well-formed, it conflicts
with what already exists. The distinction tells a client whether fixing the input
could help.
