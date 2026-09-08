# The middleware

Forty lines, and every private route goes through it.

```rust
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = auth::bearer(header)?;
    let claims = state.jwt.verify(token)?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
```

## What it does, in order

1. Read the `Authorization` header.
2. Pull the token out of `Bearer <token>`.
3. Verify the signature and expiry.
4. **Put the claims into the request extensions.**
5. Run the handler.

Any failure returns early with a 401 and the handler never runs.

## Step 4 is the interesting one

Extensions are a typed side-channel on the request: a map keyed by type. The
middleware inserts a `Claims`, and a handler asks for it:

```rust
pub async fn list_users(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    claims.require("users", "read")?;
```

The handler does not parse a header, does not know what a bearer token is, and
cannot forget to check the signature — by the time it runs, that already
happened.

There is a sharp edge: `Extension<Claims>` on a handler in the **public** router
would compile and then fail at runtime, because nothing inserted a `Claims`. The
type system does not connect "this route has the middleware" to "this handler
wants the extension". Keeping the two routers separate is what keeps that
straight.

## `and_then(|v| v.to_str().ok())`

HTTP header values are bytes, not text, and may not be valid UTF-8. A header full
of invalid bytes yields `None` here, which becomes `AuthError::Missing`, which
becomes 401.

Treating unreadable as missing is right: a token that is not text is not a token.

## Where errors become responses

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}
```

One shape for every error the API produces:

```json
{ "error": "the token has expired" }
```

The `From` implementations decide the status code, and this is where the
[401/403 distinction](errors.md) is actually made.

Database errors get special handling:

```rust
E::Db(inner) => {
    tracing::error!("database: {inner}");
    ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
}
```

The detail goes to the log; the client gets a sentence. Database errors quote
table names, column names and sometimes values, and none of that is the caller's
business — it is a free schema map for anyone probing the API.
