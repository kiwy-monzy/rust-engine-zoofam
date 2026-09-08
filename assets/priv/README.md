# Signing material

Apple Wallet Pass Type ID certificate, private key and the WWDR intermediate,
all DER. Everything here is git-ignored — a leaked `pass.key` lets anyone sign
passes as this deployment.

| File | What |
| --- | --- |
| `pass.cer` | Pass Type ID certificate (`Apple Developer Certificate.cer` also accepted) |
| `pass.key` | Private key, PKCS#8 DER (convert: `openssl pkcs8 -topk8 -outform DER -nocrypt -in pass.pem -out pass.key`) |
| `AppleWWDRCAG4.cer` | Apple WWDR intermediate (`AppleWWDRCA.cer` also accepted) |

Configured via `WALLET_PRIV_DIR` (default `priv/`, relative to the gateway's
working directory). The pass type id / team id in `.env` must match the
certificate.
