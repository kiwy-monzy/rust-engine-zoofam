# Passwords

The database never stores a password. It stores an Argon2id hash, and the
difference is what stands between a stolen database and stolen accounts.

## Hashing

```rust
pub fn hash_password(plain: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError::Hashing(e.to_string()))
}
```

The result is a PHC string carrying everything needed to check it later:

```
$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG
 └ algorithm └ version └ cost parameters └ salt └ hash
```

The salt travels with the hash. It is not a secret — its job is to make two
identical passwords hash differently, so an attacker cannot precompute one table
that cracks every account at once.

There is a test asserting exactly that:

```rust
#[test]
fn the_same_password_hashes_differently_each_time() {
    let a = hash_password("same").unwrap();
    let b = hash_password("same").unwrap();
    assert_ne!(a, b);
```

## Why Argon2 and not SHA-256

SHA-256 is designed to be fast. A modern GPU computes billions per second, which
is exactly the wrong property: a fast hash is a fast brute force.

Argon2id is designed to be slow *and memory-hungry* — the default parameters ask
for 19 MB per hash. GPUs have many cores but not many megabytes each, so the
memory cost is what removes the attacker's advantage. It won the Password Hashing
Competition in 2015 and is the current default recommendation.

Never use a general-purpose hash for passwords. If it is fast, it is wrong.

## Verifying

```rust
pub fn verify_password(plain: &str, hash: &str) -> Result<()> {
    let parsed = PasswordHash::new(hash).map_err(|_| AuthError::BadCredentials)?;
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .map_err(|_| AuthError::BadCredentials)
}
```

The stored string is parsed to recover the salt and parameters, the candidate is
hashed the same way, and the results are compared in constant time. Constant-time
comparison matters: a comparison that returns early on the first wrong byte leaks,
through timing, how much of the guess was right.

Note that a malformed hash and a wrong password produce the *same* error. A
caller cannot tell "this account has a corrupt hash" from "wrong password", and
should not be able to.

## Not saying too much

```rust
let user = controller::users::find_by_email(&state.pool, &input.email)
    .map_err(|_| ApiError::from(AuthError::BadCredentials))?;
```

An unknown email produces `NotFound`, which would naturally become a `404`. It is
deliberately converted into the same "the email or password is wrong" that a bad
password gives.

Otherwise `/auth/login` becomes a tool for discovering which addresses have
accounts: `404` means no account, `401` means there is one. That list is worth
money to a phisher.

A test asserts both paths return the identical message:

```rust
assert_eq!(
    body["error"], "the email or password is wrong",
    "an unknown email must not be distinguishable from a wrong password"
);
```

## Never sending it back

```rust
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip)]
    pub email_lower: String,
    #[serde(skip)]
    pub password_hash: String,
```

`#[serde(skip)]` means the field is not serialised, ever. `User` is returned
directly from several endpoints, and without this the hash would go out with it.

Belt and braces: a test fetches `/auth/me` and greps the response.

```rust
assert!(!text.contains("password"), "leaked: {text}");
assert!(!text.contains("argon2"), "leaked: {text}");
```

An Argon2 hash is not crackable in an afternoon, but it is crackable given a weak
password and time. It should never leave the server.

## The minimum

```rust
if input.password.chars().count() < 8 {
    return Err(Error::Invalid("the password must be at least 8 characters".into()));
}
```

`chars().count()`, not `len()`. `len()` counts **bytes**, so a password of four
emoji would pass an eight-byte check while being four characters. Neither is a
perfect measure of a password, but counting characters is at least the thing a
person thinks they typed.
