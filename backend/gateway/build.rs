fn main() {
    let frontend_dist = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../frontend/dist"));

    if !frontend_dist.exists() || !frontend_dist.join("index.html").exists() {
        println!("cargo:warning=Frontend not built, building now...");
        let status = std::process::Command::new("cmd")
            .args(["/C", "cd frontend && npm run build"])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .status();

        if let Err(e) = status {
            println!("cargo:warning=Failed to build frontend: {}", e);
        } else if !status.unwrap().success() {
            println!("cargo:warning=Frontend build failed");
        }
    }

    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");
}
