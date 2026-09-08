# A walkthrough with curl

Every command here was run against the real server. The responses are what it
actually returned.

Start with a clean database:

```bash
rm -f admin.db
cargo run -p gateway
```

## 1. It is alive

```bash
curl -s localhost:8080/health
```

```json
{ "ok": true }
```

No token needed. This is a public route.

## 2. Create an account

```bash
curl -s -X POST localhost:8080/auth/register \
  -H 'content-type: application/json' \
  -d '{"email":"me@example.com","password":"password123","display_name":"Me"}'
```

`201 Created`. The response carries the user — with no `password_hash`, because
that field is `#[serde(skip)]`.

## 3. Log in

```bash
TOKEN=$(curl -s -X POST localhost:8080/auth/login \
  -H 'content-type: application/json' \
  -d '{"email":"me@example.com","password":"password123"}' \
  | jq -r .token)
```

About 250 characters of base64. Paste it into <https://jwt.io> and read it —
it is signed, not encrypted.

## 4. The lesson

```bash
curl -s -o /dev/null -w '%{http_code}\n' localhost:8080/users
```

```
401
```

No token. *I do not know who you are.*

```bash
curl -s localhost:8080/users -H "authorization: Bearer $TOKEN"
```

```json
{ "error": "this account may not users:read" }
```

`403`. A perfectly good token — *and no*. The account exists, the password was
right, the signature verifies. It simply has no roles yet.

Confirm that:

```bash
curl -s localhost:8080/auth/me -H "authorization: Bearer $TOKEN"
```

```json
{
  "email": "me@example.com",
  "expires_at": 1787626877,
  "id": "16e6a7f7-c791-483f-b12d-1afb45e1b8e9",
  "permissions": [],
  "roles": []
}
```

Empty. That is the whole reason for the 403.

## 5. Grant a role

The `admin` role already exists — migration 2 created it, and migration 4 gave it
every permission. It just is not attached to anyone.

```bash
sqlite3 admin.db "INSERT INTO gateway_user_roles (user_id, role_id)
  SELECT u.id, r.id FROM gateway_users u, gateway_roles r
  WHERE u.email_lower='me@example.com' AND r.name='admin';"
```

Going in through SQL because the endpoint that does this needs `users:write`,
which is exactly what we do not have. Bootstrapping the first administrator
always takes one step outside the API.

## 6. Log in again

```bash
TOKEN=$(curl -s -X POST localhost:8080/auth/login \
  -H 'content-type: application/json' \
  -d '{"email":"me@example.com","password":"password123"}' | jq -r .token)
```

**A new token is required.** The old one still says `"roles": []` — permissions
are frozen at login. See [What a token carries](../auth/claims.md).

```bash
curl -s localhost:8080/auth/me -H "authorization: Bearer $TOKEN"
```

```json
{
  "email": "me@example.com",
  "permissions": ["permissions:read", "roles:read", "roles:write", "users:read", "users:write"],
  "roles": ["admin"]
}
```

Those five came out of the database by joining three tables, and are now baked
into the token.

## 7. Everything opens

```bash
curl -s -o /dev/null -w 'users:       %{http_code}\n' localhost:8080/users       -H "authorization: Bearer $TOKEN"
curl -s -o /dev/null -w 'roles:       %{http_code}\n' localhost:8080/roles       -H "authorization: Bearer $TOKEN"
curl -s -o /dev/null -w 'permissions: %{http_code}\n' localhost:8080/permissions -H "authorization: Bearer $TOKEN"
```

```
users:       200
roles:       200
permissions: 200
```

## 8. Try the other role

```bash
curl -s -X POST localhost:8080/users \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"email":"reader@example.com","password":"password123"}'
```

Give that account the `viewer` role instead, log in as it, and you get `200` on
`GET /users` and `403` on `POST /users` — read but not write. There is a test
asserting exactly that, called
`the_viewer_role_may_read_but_not_write`.
