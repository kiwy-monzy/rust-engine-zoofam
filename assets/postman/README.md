# Postman: GATEWAY-RUST

A Postman collection and environment for the `app/` RBAC gateway — the
functional API plus a full security suite (SQL injection, auth bypass,
privilege escalation, input abuse, information disclosure).

## Files

- `GATEWAY-RUST.postman_collection.json` — 81 requests in folders.
- `GATEWAY-RUST.postman_environment.json` — `baseUrl` + captured tokens/ids.

## Import

Postman → **Import** → drop both files. Pick the **GATEWAY-RUST (local)**
environment (top-right selector).

## Run

```bash
cd app
cargo run -p gateway         # starts http://127.0.0.1:8080 on SQLite
```

1. Run **0. Bootstrap** once — registers an admin, a viewer and a never-privileged
   bystander, and captures their tokens into environment variables.
2. Run any folder, or the whole collection, with the **Collection Runner**.

### Wallet folder

Covers the Apple Wallet module end to end: sample catalog, issuing a coupon
(captures `walletPassType` / `walletSerial` / `walletAuthToken`), QR, editing
`pass.json`, the public `.pkpass` download, and the pass-update web service
(register / list-updates / latest-pass / log / unregister) with its
`Authorization: ApplePass <token>` credential scheme — then delete + confirm
the download is gone. Every request tolerates **503**: when the gateway boots
without signing material in `WALLET_PRIV_DIR`, the whole module answers 503 by
design instead of failing at startup.

## Headless

The `api-pentest` Claude skill runs the whole thing with newman against a
throwaway database:

```bash
bash .claude/skills/api-pentest/scripts/run.sh
```

## The security suite asserts defence, not exploitation

A **passing** security test means the attack was correctly **refused**. SQL
injection payloads must return an ordinary `400`/`401` — never a `500`, never a
success. The DB-integrity check after the `DROP TABLE` payloads confirms nothing
executed.

| Security folder | Confirms |
|---|---|
| Auth bypass | forged / `alg:none` / blank tokens rejected; unknown path is 404 not 401 |
| Session revocation | logout really kills the JWT; logout-all invalidates every session |
| SQL injection | Diesel binds values; typed path params reject non-values before any query |
| Privilege escalation | a valid token is not permission; frozen-at-login model holds |
| Wallet abuse | viewer/no-role cannot issue; web service rejects missing/forged ApplePass tokens and refuses to accept an admin JWT as a pass credential; traversal and injection in serials are dead ends |
| Input validation | malformed/oversized/typed-wrong input is a clean 4xx, never a 500 |
| Information disclosure | errors return a sentence, not internals; DB survives |

## The role prerequisite

Bootstrap registers three accounts: **admin**, **viewer** and a **bystander**
that is never granted a role (it owns every "valid identity, zero permissions"
assertion). A freshly registered account holds no role, so the Users/Roles/
Permissions/Wallet folders assert `200 OR 403`. To make them 200 and enable
the viewer tests, grant roles in the DB, then re-run **0. Bootstrap → Login**:

```bash
sqlite3 app/admin.db "INSERT INTO gateway_user_roles (user_id, role_id)
  SELECT u.id, r.id FROM gateway_users u, gateway_roles r
  WHERE (u.email_lower='admin.pm@example.com' AND r.name='admin')
     OR (u.email_lower='viewer.pm@example.com' AND r.name='viewer');"
```

Permissions are frozen into the token at login, so a new login is required after
granting — which the `Self-granted admin role is still refused` test relies on.

## Scope

Everything targets `127.0.0.1`. This is authorised testing of your own gateway.
Do not repoint `baseUrl` at a host you do not own.
