//! Generates one sample pass per Wallet style, each with its own artwork, plus
//! a scannable gallery page.
//!
//! Run:
//!     cargo run -p applewallet --bin make-samples -- --base-url http://192.168.1.20:8000
//!
//! Signing needs the Pass Type ID certificate and key in `priv/`. Without them
//! this still writes every `pass.json` and image set so you can inspect the
//! output — it just cannot produce an installable `.pkpass`, because Wallet
//! refuses anything unsigned.

use std::fs;
use std::path::{Path, PathBuf};

use applewallet::{samples, PassKit, WalletConfig};

struct Args {
    base_url: String,
    out: PathBuf,
    assets_root: PathBuf,
    priv_dir: PathBuf,
}

fn parse_args() -> Args {
    let mut base_url = "http://127.0.0.1:8000".to_string();
    let mut out = PathBuf::from("out/passes");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut assets_root = manifest.join("assets/samples");
    let mut priv_dir = manifest.join("priv");

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let next = || argv.get(i + 1).cloned().unwrap_or_default();
        match argv[i].as_str() {
            "--base-url" => {
                base_url = next();
                i += 1;
            }
            "--out" => {
                out = PathBuf::from(next());
                i += 1;
            }
            "--assets" => {
                assets_root = PathBuf::from(next());
                i += 1;
            }
            "--priv" => {
                priv_dir = PathBuf::from(next());
                i += 1;
            }
            "-h" | "--help" => {
                println!(
                    "make-samples [--base-url URL] [--out DIR] [--assets DIR] [--priv DIR]"
                );
                std::process::exit(0);
            }
            other => eprintln!("ignoring unknown argument {other}"),
        }
        i += 1;
    }

    Args {
        base_url: base_url.trim_end_matches('/').to_string(),
        out,
        assets_root,
        priv_dir,
    }
}

fn qr_svg(data: &str) -> String {
    use qrcode::render::svg;
    use qrcode::QrCode;
    match QrCode::new(data.as_bytes()) {
        Ok(code) => code
            .render::<svg::Color>()
            .min_dimensions(190, 190)
            .quiet_zone(true)
            .build(),
        Err(e) => format!("<p>QR failed: {e}</p>"),
    }
}

/// Picks the image the gallery should show for a style.
fn hero_for(style: &str) -> &'static str {
    match style {
        "eventTicket" => "background",
        "coupon" | "storeCard" => "strip",
        "generic" => "thumbnail",
        _ => "logo",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    fs::create_dir_all(&args.out)?;

    let catalog = samples::sample_catalog();

    // Signing material is optional; report clearly which mode we are in rather
    // than failing, because the unsigned output is still useful for review.
    let can_sign = PassKit::new(WalletConfig {
        priv_dir: args.priv_dir.clone(),
        ..WalletConfig::default()
    })
    .is_ok();

    if can_sign {
        println!("signing with certificates in {}", args.priv_dir.display());
    } else {
        println!(
            "no usable certificates in {} - writing pass.json + images only.\n\
             Wallet will not install an unsigned pass; see priv/README.md.",
            args.priv_dir.display()
        );
    }

    let mut cards = String::new();

    for entry in &catalog {
        let Some(pass) = samples::sample_pass_by_id(&entry.id) else {
            eprintln!("no builder for catalog id {}", entry.id);
            continue;
        };

        let assets_dir = args.assets_root.join(&entry.id);
        if !assets_dir.is_dir() {
            eprintln!(
                "missing artwork for {} - run tools/make_images.py first",
                entry.id
            );
        }

        let kit = PassKit::new(WalletConfig {
            priv_dir: args.priv_dir.clone(),
            assets_dir: Some(assets_dir.clone()),
            base_url: args.base_url.clone(),
            ..WalletConfig::default()
        });

        let status: String;

        match kit {
            Ok(kit) => match kit.sign_pass(&pass) {
                Ok(bytes) => {
                    let path = args.out.join(format!("{}.pkpass", entry.id));
                    fs::write(&path, &bytes)?;
                    status = format!("{} bytes", bytes.len());
                    println!("  {:<18} {:>9}  {}", entry.id, status, path.display());
                }
                Err(e) => {
                    status = format!("sign failed: {e}");
                    eprintln!("  {:<18} {status}", entry.id);
                }
            },
            Err(_) => {
                // Unsigned fallback: the pass body plus its artwork, so the
                // content can still be reviewed.
                let dir = args.out.join(format!("{}-unsigned", entry.id));
                fs::create_dir_all(&dir)?;
                fs::write(dir.join("pass.json"), serde_json::to_vec_pretty(&pass)?)?;
                copy_dir(&assets_dir, &dir)?;
                status = "unsigned".into();
                println!("  {:<18} {:>9}  {}", entry.id, status, dir.display());
            }
        }

        let url = format!("{}/{}.pkpass", args.base_url, entry.id);
        let hero = hero_for(&entry.pass_style);
        cards.push_str(&format!(
            r#"<article class="card">
  <div class="hero" style="background-image:url('../../crates/applewallet/assets/samples/{id}/{hero}@2x.png');background-color:{colour}"></div>
  <div class="meta">
    <h2>{name}</h2>
    <p class="style">{style}</p>
    <p class="desc">{desc}</p>
    <p class="status">{status}</p>
  </div>
  <div class="qr">{qr}<a href="{url}">{url}</a></div>
</article>
"#,
            id = html_escape(&entry.id),
            hero = hero,
            colour = html_escape(&entry.colour),
            name = html_escape(&entry.name),
            style = html_escape(&entry.pass_style),
            desc = html_escape(&entry.description),
            status = html_escape(&status),
            qr = qr_svg(&url),
            url = html_escape(&url),
        ));
    }

    let page = format!(
        r#"<!doctype html>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Wallet sample passes</title>
<style>
  :root {{ color-scheme: light dark; }}
  body {{ font: 15px/1.5 ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
         margin: 0; padding: 32px 20px 64px; background: Canvas; color: CanvasText; }}
  .wrap {{ max-width: 1000px; margin: 0 auto; }}
  h1 {{ font-size: 22px; margin: 0 0 4px; }}
  .lede {{ color: color-mix(in srgb, CanvasText 62%, Canvas); margin: 0 0 28px; }}
  .grid {{ display: grid; gap: 16px; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); }}
  .card {{ border: 1px solid color-mix(in srgb, CanvasText 16%, Canvas); border-radius: 14px;
          overflow: hidden; background: color-mix(in srgb, CanvasText 3%, Canvas); }}
  .hero {{ height: 116px; background-size: cover; background-position: center; }}
  .meta {{ padding: 14px 16px 6px; }}
  .meta h2 {{ font-size: 16px; margin: 0 0 2px; }}
  .style {{ font: 600 11px ui-monospace, SFMono-Regular, Menlo, monospace;
           text-transform: uppercase; letter-spacing: .06em;
           color: color-mix(in srgb, CanvasText 55%, Canvas); margin: 0 0 8px; }}
  .desc {{ margin: 0 0 8px; font-size: 13.5px; }}
  .status {{ margin: 0; font-size: 12px; color: color-mix(in srgb, CanvasText 50%, Canvas); }}
  .qr {{ padding: 6px 16px 18px; display: flex; flex-direction: column; align-items: center; gap: 6px; }}
  .qr svg {{ width: 190px; height: 190px; }}
  .qr a {{ font: 11px ui-monospace, SFMono-Regular, Menlo, monospace; word-break: break-all;
          text-align: center; color: color-mix(in srgb, CanvasText 60%, Canvas); }}
  .note {{ border: 1px solid color-mix(in srgb, CanvasText 18%, Canvas); border-radius: 10px;
          padding: 12px 16px; margin: 0 0 24px; font-size: 13.5px;
          background: color-mix(in srgb, CanvasText 4%, Canvas); }}
  code {{ font: 12.5px ui-monospace, SFMono-Regular, Menlo, monospace; }}
</style>
<div class="wrap">
  <h1>Wallet sample passes</h1>
  <p class="lede">One sample per style, each with its own artwork. Scan a code with an iPhone to add the pass.</p>
  <p class="note">
    Scanning only works if this machine serves the passes at <code>{base}</code> and the
    phone is on the same Wi-Fi. From <code>{out}</code> run:<br>
    <code>python serve.py</code><br>
    Use that rather than <code>python -m http.server</code>: the built-in server sends
    <code>.pkpass</code> as <code>application/octet-stream</code>, so Safari saves it as a
    file and Wallet never opens. <code>serve.py</code> sends
    <code>application/vnd.apple.pkpass</code>.
  </p>
  <div class="grid">
{cards}  </div>
</div>
"#,
        base = html_escape(&args.base_url),
        out = html_escape(&args.out.display().to_string()),
        cards = cards,
    );

    let index = args.out.join("index.html");
    fs::write(&index, page)?;
    println!("\ngallery -> {}", index.display());
    Ok(())
}

/// Copies the `.png` files of `from` into `to`. Missing source is not an error.
fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    let Ok(entries) = fs::read_dir(from) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "png") {
            if let Some(name) = path.file_name() {
                fs::copy(&path, to.join(name))?;
            }
        }
    }
    Ok(())
}
