# Public and private

This is the core of the whole project, and it is two functions.

```rust
pub fn public(state: AppState) -> Router {
    Router::new()
        .route("/health", get(controllers::health))
        .route("/auth/register", post(controllers::register))
        .route("/auth/login", post(controllers::login))
        .with_state(state)
}

pub fn private(state: AppState) -> Router {
    Router::new()
        .route("/auth/me", get(controllers::me))
        .route("/users", get(controllers::list_users).post(controllers::create_user))
        // ... the rest
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::require_auth,
        ))
        .with_state(state)
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .merge(public(state.clone()))
        .merge(private(state))
        .layer(TraceLayer::new_for_http())
}
```

Two routers. The private one has `require_auth` attached; the public one does
not. `app()` merges them into the thing that gets served.

## Why this shape

A route is private **because of which function it was registered in**. Not
because of a naming convention, not because of a list of exempt paths kept
somewhere else, not because someone remembered to add an attribute.

Compare the usual alternative — one router and a middleware that checks the path:

```rust
if path.starts_with("/auth/login") || path == "/health" { skip_auth() }
```

Every new public route means editing that condition. Every typo is a security
hole that no test will catch, because the route still works — it is just
unprotected. And `/health` versus `/healthz` is a very quiet bug.

Here, moving a route between `public()` and `private()` is the entire change, and
forgetting to protect a route means you wrote it in the wrong function, which is
visible in a diff.

## `route_layer`, not `layer`

This is the detail worth remembering.

- **`.layer()`** applies to every request the router sees, including ones that
  match no route.
- **`.route_layer()`** applies only after a route has matched.

With `.layer()` on the private router, a request to a URL that does not exist
would run `require_auth` first and answer **401** instead of **404**.

That sounds harmless. It is not: it turns the API into an oracle. An attacker
probing paths gets 401 for everything, except the paths that do not exist behind
this router — so any deviation from 401 is a signal about what exists. Worse, it
is confusing for legitimate clients: a typo in a URL reports an auth problem.

`TraceLayer` in `app()` deliberately uses `.layer()` — logging every request,
including the 404s, is exactly what you want from logging.

## `with_state` comes last

`Router::with_state` fixes the state type. Call it before adding routes that need
state and the types will not line up. Both functions add every route first, then
attach state at the end.

`from_fn_with_state` needs its own clone of the state because the middleware runs
outside the handler and gets its `State` separately.

## Register is public on purpose

`/auth/register` being open is what lets you create the first account on an empty
database. It is a choice, not a rule.

To close it, move that one line from `public()` to `private()`. `POST /users`
already does the same job for a signed-in administrator, so nothing is lost
except self-service signup.
