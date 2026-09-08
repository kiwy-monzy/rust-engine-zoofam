# Every route

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
| POST | `/users/{id}/roles/{role_id}` | private | `users:write` |
| DELETE | `/users/{id}/roles/{role_id}` | private | `users:write` |
| GET | `/roles` | private | `roles:read` |
| POST | `/roles` | private | `roles:write` |
| GET | `/roles/{id}` | private | `roles:read` |
| DELETE | `/roles/{id}` | private | `roles:write` |
| POST | `/roles/{id}/permissions/{permission_id}` | private | `roles:write` |
| DELETE | `/roles/{id}/permissions/{permission_id}` | private | `roles:write` |
| GET | `/permissions` | private | `permissions:read` |
| POST | `/permissions` | private | `roles:write` |
| DELETE | `/permissions/{id}` | private | `roles:write` |

## Reading the table

**Three public routes.** `/health` so a load balancer can check the process is
alive without a credential. `/auth/register` and `/auth/login` because you cannot
require a token from someone who is trying to get one.

**`/auth/me` needs a token but no permission.** Any authenticated user may ask
who they are. It is the endpoint to hit when debugging: it shows exactly what
your token claims.

**Writing permissions is under `roles:write`, not `permissions:write`.** A
`permissions:read` exists but there is no matching write. Creating a permission
is an administrative act of the same weight as editing a role, and inventing a
second permission for it would be one more thing to grant and forget. This is a
design choice you might make differently.

## The bodies

```
POST /auth/register   { "email", "password", "display_name"?, "avatar_url"? }
POST /auth/login      { "email", "password" }
POST /users           same as register
PATCH /users/{id}     { "display_name"?, "avatar_url"?, "is_active"? }
POST /roles           { "name", "description"? }
POST /permissions     { "module", "action", "description"? }
```

`?` marks optional. `PATCH` fields are all optional and only the ones present
are changed — that is what makes it a PATCH rather than a PUT.

## Assign and revoke

```
POST   /users/{id}/roles/{role_id}
DELETE /users/{id}/roles/{role_id}
```

No body. The URL says everything: this user, that role. Both answer `204 No
Content`, and both are idempotent — assigning twice is not an error, because the
insert uses `on_conflict_do_nothing`, and revoking a role nobody has deletes
nothing and reports success.

Idempotence is worth having on relationship endpoints. A client that retries
after a timeout should not get an error for succeeding twice.
