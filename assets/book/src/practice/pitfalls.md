# Two bugs worth knowing

Both were hit while building this. Both are easy to hit again, and neither
announces itself clearly.

## In-memory SQLite gives each connection its own database

The tests run with `DATABASE_URL=:memory:`. Ten of twelve passed. Two failed:

```
called `Result::unwrap()` on an `Err` value:
  DatabaseError(Unknown, "no such table: gateway_role_permissions")
```

But a different test asserted all five migrations applied, and it passed.

**Both were true.** With SQLite, `:memory:` does not name a database — it means
"give me a fresh private one". Every connection gets its own. The pool opened one
connection, ran the migrations into it, and later handed out a *second*
connection, which was a brand new empty database.

The tests that passed happened to reuse the one connection. The tests that failed
caused a second to be opened.

The fix:

```rust
#[cfg(feature = "sqlite")]
fn pool_size(url: &str) -> u32 {
    if url.contains(":memory:") { 1 } else { 8 }
}
```

One connection means one database.

File-backed SQLite and PostgreSQL do not have this problem — every connection
opens the same thing. It is specific to in-memory, which is exactly where tests
live.

The alternative is `file::memory:?cache=shared`, which lets connections share one
in-memory database. That shares it across the whole *process*, so parallel tests
would then see each other's data. Capping the pool is the better trade here.

## Nested checkouts deadlock a one-connection pool

Fixing the first bug would have created a worse one.

```rust
pub fn list(pool: &DbPool) -> Result<Vec<UserWithRoles>> {
    let mut c = conn(pool)?;                     // holds connection 1
    let users = ...load(&mut c)?;
    for user in users {
        let roles = role_names(pool, &user.id)?; // asks for connection 2
    }
}
```

`role_names` takes `&DbPool` and checks out its own connection. With a pool of
eight this works — wastefully, but it works. With a pool of one it **hangs
forever**: `list` holds the only connection and waits for `role_names`, which
waits for `list` to give it back.

Not a crash. Not an error. The request simply never answers.

The fix is a habit worth keeping at any pool size — functions that need a
connection take one, rather than fetching their own:

```rust
pub fn list(pool: &DbPool) -> Result<Vec<UserWithRoles>> {
    let mut c = conn(pool)?;
    let users = ...;
    for user in users {
        let roles = roles_on(&mut c, &user.id)?;   // reuses it
    }
}

pub fn role_names(pool: &DbPool, user_id: &str) -> Result<Vec<String>> {
    let mut c = conn(pool)?;
    roles_on(&mut c, user_id)                      // public entry point
}

fn roles_on(c: &mut DbConn, user_id: &str) -> Result<Vec<String>> { ... }
```

The private function does the work and borrows a connection. The public one
checks out and delegates. Callers that already hold a connection use the private
path.

**One request, one connection.** A pool of eight serving requests that each take
two connections is really a pool of four, and it will deadlock under load rather
than in testing — which is much more expensive to discover.

`controller::roles::list` had the identical bug and the identical fix.

## The pattern behind both

Neither is a Rust problem or a Diesel problem. Both come from a resource that
looks shared but is not, and from code that assumes taking a resource is free.

They found each other: fixing the pool size turned a silent wrong-data bug into a
hang, which is how the second surfaced. A larger pool would have hidden both
until production.
