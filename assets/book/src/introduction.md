# Introduction

This is a working HTTP API with users, roles, permissions and JWT authentication.
It is about 900 lines of Rust across six crates, and every one of those crates
exists to answer exactly one question.

It was written to be **read**, not just run. The source has no comments at all —
this book is where the explanation lives. If you want to know why a line is
there, look it up here.

## What it does

- Registers users and signs them in
- Hands out a signed token that says who you are and what you may do
- Refuses requests that have no valid token
- Refuses requests that have a valid token but insufficient permission
- Keeps users, roles and permissions in a real database, created by real
  migrations

## What you will learn

| Topic | Where |
|---|---|
| Diesel migrations, one table per file | [Migrations](database/migrations.md) |
| Supporting SQLite and PostgreSQL from one codebase | [Two backends](database/backends.md) |
| Hashing passwords so a database leak is not a password leak | [Passwords](auth/passwords.md) |
| Signing and verifying JWTs | [JSON Web Tokens](auth/jwt.md) |
| Splitting routes into public and private | [Public and private](routing/split.md) |
| Why 401 and 403 are not the same answer | [401 and 403](routing/errors.md) |

## How to read it

Front to back if the whole thing is new. If you only came for one topic, the
chapters stand alone.

Every claim in this book is checked by a test. Where a chapter says "this
happens", there is a test asserting it, and the test is named. Run them:

```bash
cargo test --workspace
```

Twenty tests, no database server required.
