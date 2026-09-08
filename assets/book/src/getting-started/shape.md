# The shape of the project

Six crates. Each arrow is a dependency, and nothing points backwards.

```
gateway  ──▶  routes  ──▶  controller  ──▶  models  ──▶  db
                 └─────▶  auth
```

That direction is the whole design. You can read any crate knowing it cannot
reach into the ones above it, so `db` cannot accidentally start caring about HTTP
status codes and `auth` cannot start running queries.

| Crate | Answers | Knows about |
|---|---|---|
| `db` | how do I get a connection, and are migrations applied? | Diesel, the pool |
| `models` | what shapes live in the database? | tables and their Rust structs |
| `controller` | what can be done with those shapes? | queries, validation |
| `auth` | who is this, and may they? | passwords and tokens |
| `routes` | which URL runs which of those? | HTTP, the public/private split |
| `gateway` | wire it up and listen | almost nothing |

## The one worth staring at

`auth` does not depend on `db`.

It never loads a user, never opens a connection, never sees a table. It hashes a
password, checks a password against a hash, signs a token and reads one back.
That is all.

So where does signing in happen? Half in `controller::users` (find the user by
email, read their roles) and half in `auth` (check the password, sign the token).
`routes::controllers::login` is the only place the two halves meet.

This is why `auth` has seven unit tests that need no database at all, and why you
could lift the whole crate into another project unchanged.

## Why `gateway` is tiny

Forty lines. It loads `.env`, starts logging, creates the pool, runs migrations,
builds the router, and serves it.

If your `main.rs` is short, the interesting decisions are somewhere you can name.
If it is long, they are nowhere.

## The dependency you should question

`controller` depends on `auth`, which looks backwards — why does a database layer
know about passwords?

Because `controller::users::create` hashes the password before it inserts. The
alternative is accepting an already-hashed password, which means every caller has
to remember to hash, and one that forgets stores a plaintext password with no
error. Making it impossible to insert an unhashed password is worth the arrow.
