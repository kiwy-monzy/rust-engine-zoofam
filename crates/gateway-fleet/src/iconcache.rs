//! Downloading source-supplied marker icons, once.
//!
//! Sources advertise their artwork as absolute URLs on their own CDNs. The map
//! client cannot fetch those directly — the browser blocks a cross-origin image
//! fetch that carries no CORS headers, and none of these CDNs send them. So the
//! gateway downloads each icon once and hands the client a `data:` URI, which
//! needs no second request and no cross-origin permission at all.
//!
//! Cached by URL for the process lifetime. Icon artwork is effectively static —
//! a taxi type's PNG does not change between polls — and there are a couple of
//! dozen of them per zone, so a bounded in-memory map is the right size of
//! solution. A restart re-fetches, which costs one small request per type.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use base64::Engine;

/// Largest icon we will inline. These are 32–128 px PNGs; anything much bigger
/// is not an icon, and inlining it would bloat every marker payload.
const MAX_ICON_BYTES: usize = 256 * 1024;

fn cache() -> &'static Mutex<BTreeMap<String, Option<String>>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, Option<String>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Looks up a URL in the cache. `Some(None)` means "tried and failed" — that is
/// remembered too, so a broken URL is not retried on every poll.
fn cached(url: &str) -> Option<Option<String>> {
    cache().lock().ok()?.get(url).cloned()
}

fn remember(url: &str, value: Option<String>) {
    if let Ok(mut c) = cache().lock() {
        c.insert(url.to_string(), value);
    }
}

/// Turns `id → URL` into `id → data: URI`, downloading whatever is not cached.
///
/// Entries that cannot be fetched are dropped rather than passed through as
/// URLs: a bare CDN link would fail in the browser anyway, and a marker with no
/// icon falls back to its coloured shape, which is a working outcome.
pub async fn resolve(urls: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (id, url) in urls {
        // Already a data URI (some sources inline their own) — nothing to do.
        if url.starts_with("data:") {
            out.insert(id.clone(), url.clone());
            continue;
        }
        if let Some(hit) = cached(url) {
            if let Some(uri) = hit {
                out.insert(id.clone(), uri);
            }
            continue;
        }
        match download(url).await {
            Some(uri) => {
                remember(url, Some(uri.clone()));
                out.insert(id.clone(), uri);
            }
            None => remember(url, None),
        }
    }
    out
}

async fn download(url: &str) -> Option<String> {
    let resp = reqwest::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png")
        .split(';')
        .next()
        .unwrap_or("image/png")
        .to_string();
    // Only images. A CDN that answers an icon URL with an HTML error page
    // should not have that page inlined into every marker payload.
    if !mime.starts_with("image/") {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    if bytes.is_empty() || bytes.len() > MAX_ICON_BYTES {
        return None;
    }
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn data_uris_pass_through_untouched() {
        let mut urls = BTreeMap::new();
        urls.insert("1".to_string(), "data:image/png;base64,AAAA".to_string());
        let out = resolve(&urls).await;
        assert_eq!(
            out.get("1").map(String::as_str),
            Some("data:image/png;base64,AAAA")
        );
    }

    /// A failed download must be remembered, or every poll retries a URL that
    /// is never going to work.
    #[tokio::test]
    async fn a_failed_download_is_cached_as_a_failure() {
        let url = "http://127.0.0.1:1/never-listening.png";
        let mut urls = BTreeMap::new();
        urls.insert("9".to_string(), url.to_string());

        let out = resolve(&urls).await;
        assert!(
            out.is_empty(),
            "a failed icon must be dropped, not passed through"
        );
        assert_eq!(cached(url), Some(None), "the failure was not remembered");

        // Second pass is served from the cache — still empty, no second attempt.
        assert!(resolve(&urls).await.is_empty());
    }

    #[tokio::test]
    async fn an_empty_map_resolves_to_an_empty_map() {
        assert!(resolve(&BTreeMap::new()).await.is_empty());
    }
}
