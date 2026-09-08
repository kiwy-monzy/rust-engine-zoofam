# applewallet

Apple Wallet (`.pkpass`) generator, signer, and pass-update web service with APNs
push — a reusable Rust library extracted from the ehealth-apple-wallet project
and mounted into **mvt-server**'s admin routes.

## What it does

- Build any of the five Wallet pass styles (boarding pass, event ticket, coupon,
  store card, generic) with a typed model, or edit `pass.json` directly.
- Package + **PKCS#7 detached-sign** a real `.pkpass` (OpenSSL), using the Pass
  Type ID certificate + WWDR intermediate.
- Issue **updatable** passes: stamp a `webServiceURL` + per-pass auth token, store
  the body, and re-sign on demand.
- Implement the Apple Wallet **pass-update web service** (device register /
  unregister / list-updated / get-latest / log) via a pluggable `PassStore`.
- **Push updates** to registered devices over HTTP/2 to `api.push.apple.com`
  using the Pass Type ID certificate as the APNs client identity.
- Render an **SVG QR code** for a LAN download URL so an iPhone can scan-to-add.

## Signing material (`priv/`)

Place these (DER) in the configured `priv_dir` (repo default: `./priv`):

| File | What |
|---|---|
| `pass.cer` *or* `Apple Developer Certificate.cer` | Pass Type ID certificate |
| `pass.key` | private key, **PKCS#8 DER** (`openssl pkcs8 -topk8 -outform DER -nocrypt`) |
| `AppleWWDRCAG4.cer` *or* `AppleWWDRCA.cer` | Apple WWDR intermediate |

`cer.bat` in the ehealth project generates the key + CSR. The directory is
git-ignored — never commit certs/keys.

## Quick start

```rust
use applewallet::{PassKit, WalletConfig, samples};

let kit = PassKit::new(WalletConfig {
    base_url: "http://192.168.1.20:5887".into(), // a LAN IP the iPhone can reach
    ..WalletConfig::default()
})?;

// One-off signed pass:
std::fs::write("event.pkpass", kit.sign_pass(&samples::sample_event_ticket())?)?;

// Updatable pass + later push:
let issued = kit.issue_updatable(&samples::sample_store_card())?;
kit.update_pass_json(&issued.pass_type, &issued.serial, new_json_bytes)?;
kit.push_update(&issued.pass_type, &issued.serial).await?;   // APNs nudge

// QR for scan-to-add:
let svg = kit.qr_svg(&format!("{}/wallet/pass/{}.pkpass", kit.config.base_url, issued.serial))?;
```

## In mvt-server

- **Admin UI:** `/admin/wallet` — sample gallery with per-pass QR + LAN download,
  `pass.json` preview/edit, re-sign.
- **Public (iPhone-reachable):**
  - `GET /wallet/sample/{id}.pkpass`, `GET /wallet/pass/{serial}.pkpass`
  - `…/wallet/v1/devices/{deviceId}/registrations/{passType}/{serial}` (POST/DELETE)
  - `GET …/wallet/v1/devices/{deviceId}/registrations/{passType}?passesUpdatedSince=`
  - `GET …/wallet/v1/passes/{passType}/{serial}`
  - `POST …/wallet/v1/log`

The base URL is auto-detected from the primary LAN IP at startup so QR codes and
`webServiceURL` resolve from a phone on the same Wi-Fi.

## Notes / limits

- The in-memory `PassStore` is fine for dev; implement the trait over a DB for
  production persistence of passes + device registrations.
- APNs uses certificate auth (the pass cert). The device must be on a network
  that can reach `api.push.apple.com`, and the pass must already be installed
  (so it has registered a push token) before updates land.
