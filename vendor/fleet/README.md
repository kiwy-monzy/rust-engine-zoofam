# fleet

One library for every live source the gateway follows.

Bolt taxis, MarineTraffic vessels, FlightRadar24 aircraft and Tanzania SGR
trains used to be four separate crates — four error types, four HTTP clients,
four ideas of what a tracked thing is, and a fifth crate that wrapped three of
them. They are four modules here, behind one trait.

## Layout

```
src/
  error.rs      Error, Result — one error type, classified by what a caller can do about it
  model.rs      Vehicle, Kind — the one shape every source produces
  wire.rs       the same model as protobuf, generated from protos/fleet
  source.rs     the Source trait: Credential, Area, Catch
  sources.rs    the four implementations, and all()
  grid.rs       H3 binning, over the vendored pure-Rust h3o
  bolt/         rider API: phone OTP, nearby vehicles, categories, icons
  marine/       AIS marker geometry, ship types, and the MarineTraffic tile feed
  flight/       FlightRadar24's gRPC-web feed, airports, searches
  sgr/          TRC TICIDIS timetables, coaches, seat maps
protos/
  fleet/        this library's own contract — what every source normalises to
  flight/       FlightRadar24's schema (upstream's)
  marine/       MarineTraffic's mobile tile schema (upstream's)
```

## The trait

```rust
#[async_trait]
pub trait Source: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn kind(&self) -> Kind;
    fn credential(&self) -> Credential;
    async fn fetch(&self, area: Area, secret: Option<&str>) -> Result<Catch>;
}
```

A caller walks `fleet::all()` and never learns any source's name. Adding one is
a registration, not another arm in three separate `match`es.

`Credential` is how a source says what it needs: `None` for public feeds, a
`Cookie` an operator pastes, or a `Session` the caller mints and keeps. Nothing
here stores a secret and nothing here has one compiled in.

## Features

| Feature | Pulls in | For |
|---|---|---|
| `geometry` | nothing | vessel marker maths — compiles to WASM |
| `net` | reqwest, tokio | shared plumbing for anything networked |
| `bolt` `marine` `flight` `sgr` | `net`, plus each source's needs | one source each |

Default is all four. A map client that only draws ship outlines takes:

```toml
fleet = { path = "../vendor/fleet", default-features = false, features = ["geometry"] }
```

which is why the geometry lives apart from the feed that fills it.

## What was deliberately removed

* **A captured FlightRadar24 device id**, compiled into every request header. It
  is somebody's browser fingerprint and it pinned every deployment to one
  session. Now `Config::device_id`, empty by default and sent only when set.
* **~750 lines of hardcoded airports** keyed by country code. They went stale on
  their own schedule; `flight::airports` asks the live API.
* **A country's worth of hardcoded searches** — `search_air_tanzania`,
  `search_dar_es_salaam`, `search_moroni`. Which airline and which airport are
  an application's business, so they are parameters now.
* **Duplicate and dead crates**: a second `bolt` package with no `src/` at all,
  a `marinetraffic` crate overlapping `marineradar`, an unused `fleet-lib`, four
  stray `Cargo.lock` files, two `desktop.ini`s, and an `index.html` living
  inside a protocol client.

## Notes

* Edition 2024, `#![forbid(unsafe_code)]`.
* rustls only, never native-tls — the gateway links this into a static musl
  binary, and keeping one TLS stack stops cargo unifying two into a build.
* `Error` distinguishes `MissingCredential`, `Blocked` and `RateLimited` rather
  than flattening them to one string. A blocked fetch that read as an empty
  result is the bug that motivated it: an operator stared at a map with no ships
  and no error for weeks.
