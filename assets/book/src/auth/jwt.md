# JSON Web Tokens

A JWT is a signed statement. The server says "this is who they are and what they
may do", signs it, and hands it to the client. The client sends it back on every
request, and the server checks the signature instead of the database.

## Three parts

```
eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxNmU2YTdmNyIsImV4cCI6MTc4NzYyNjg3N30.dBjftJeZ4CVP
└────── header ──────┘└──────────────── payload ────────────────┘└─── signature ───┘
```

Header and payload are base64url-encoded JSON, joined by dots, with a signature
over both.

**Base64 is not encryption.** Anyone holding a token can read its payload —
paste one into <https://jwt.io> and see. The signature does not hide the contents,
it proves nobody changed them. Never put a secret in a token.

## Signing

```rust
pub fn issue(&self, user_id: &str, email: &str, roles: Vec<String>, perms: Vec<String>) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        roles,
        perms,
        iat: now.timestamp(),
        exp: (now + self.ttl).timestamp(),
    };
    encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
}
```

HS256 is HMAC-SHA256: one shared secret both signs and verifies. Simple, fast,
and right when the same service does both — which is the case here.

The alternative, RS256, signs with a private key and verifies with a public one,
so other services can verify without being able to mint. Reach for that when you
have several services; it is unnecessary complexity for one.

## Verifying

```rust
pub fn verify(&self, token: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims = HashSet::from(["exp".to_string()]);
    decode::<Claims>(token, &self.decoding, &validation)
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            ExpiredSignature => AuthError::Expired,
            _ => AuthError::Invalid(e.to_string()),
        })
}
```

`Validation::new(Algorithm::HS256)` is doing security work that is easy to miss.
It pins the algorithm. Without pinning, a decoder may believe the token's own
header — and a token claiming `"alg": "none"` would then verify with no signature
at all. That is a real, named family of JWT vulnerabilities.

Requiring `exp` closes the matching hole: a token with no expiry claim would
otherwise be valid forever.

Two tests pin this behaviour:

```rust
fn a_token_signed_with_another_secret_is_rejected()
fn an_expired_token_is_rejected_as_expired()
```

The second distinguishes expired from invalid, because they mean different things
to a client: expired means "log in again", invalid means "something is wrong".

## Reading the header

```rust
pub fn bearer(header: Option<&str>) -> Result<&str> {
    let value = header.ok_or(AuthError::Missing)?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .ok_or(AuthError::Missing)?
        .trim();
    if token.is_empty() {
        return Err(AuthError::Missing);
    }
    Ok(token)
}
```

Both capitalisations, because the scheme is case-insensitive and clients differ.

The empty check is the one that matters. `Authorization: Bearer ` with nothing
after it would otherwise produce an empty token, and an empty token is not a
missing one — it would take a different path through the code for no good reason.
There is a test for the blank case specifically.

## The secret

```rust
let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
```

Anyone holding this string can mint a token for any user with any permission. It
is the most sensitive value in the system.

The fallback exists so `cargo run` works with no setup. **Change it in
production, and keep it out of git.** Changing it also invalidates every existing
token at once, which is the emergency lever if one leaks.
