fn main() {
    tauri_build::build();

    // Auto-build napi-rs native addon
    #[cfg(feature = "napi")]
    {
        napi_rs_build::Builder::default()
            .name("gateway_addon")
            .target(true)
            .build();
    }
}
