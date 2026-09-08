# Signing material

This directory is **empty on purpose**. It holds the Apple Pass Type ID
certificate and private key, and those were deliberately not copied in when this
crate was vendored — a leaked `pass.key` lets anyone sign passes as you, and
this repository has files staged for commit.

Everything here except this README and `.gitignore` is git-ignored.

## What goes here

All DER format:

| File | What |
| --- | --- |
| `pass.cer` *or* `Apple Developer Certificate.cer` | Pass Type ID certificate |
| `pass.key` | Private key, PKCS#8 DER |
| `AppleWWDRCAG4.cer` *or* `AppleWWDRCA.cer` | Apple WWDR intermediate |

Convert a PEM key if needed:

```bash
openssl pkcs8 -topk8 -outform DER -nocrypt -in pass.pem -out pass.key
```

## Using the certificates you already have

They live in the source project this crate came from. Rather than copying them
here, point the tool at them:

```bash
cargo run --bin make-samples -- --priv "G:/Github2026/New folder (2)/crates/applewallet/priv"
```

That is how the sample passes in `out/passes/` were generated — the keys never
moved.

## Without them

`make-samples` still runs. It writes each `pass.json` plus its artwork to
`<id>-unsigned/` so the content can be reviewed, but produces no `.pkpass`:
Wallet refuses to install an unsigned pass, so emitting one would only look like
it worked.
