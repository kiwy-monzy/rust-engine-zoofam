//! Compiles the protobuf schemas under `protos/`.
//!
//! Three groups, and they are separate for a reason:
//!
//! * `protos/fleet` — this library's own contract, the one shape every source
//!   normalises to. Always compiled: it is the domain model.
//! * `protos/flight` — FlightRadar24's gRPC-web schema, upstream's.
//! * `protos/marine` — MarineTraffic's mobile tile schema, upstream's.
//!
//! Only the `flight` feature needs the upstream sets, and cargo has no way to
//! make a build-dependency optional, so the work is skipped by feature rather
//! than the dependency being dropped.

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=protos");

    // Use the vendored protoc so no system install is required.
    if let Ok(protoc) = protoc_bin_vendored::protoc_bin_path() {
        // SAFETY: single-threaded build script, before anything reads the
        // environment. `set_var` became unsafe in edition 2024 because of the
        // races it invites in threaded programs; there are no threads here.
        unsafe {
            std::env::set_var("PROTOC", protoc);
        }
    }

    // The domain model. Serde derives so a catch can go straight to JSON for
    // clients that would rather not speak protobuf.
    if std::env::var_os("CARGO_FEATURE_PROTO").is_none() {
        return Ok(());
    }
    let mut fleet = prost_build::Config::new();
    fleet.type_attribute(".fleet", "#[derive(serde::Serialize, serde::Deserialize)]");
    fleet.compile_protos(&["protos/fleet/fleet.proto"], &["protos/fleet/"])?;

    if std::env::var_os("CARGO_FEATURE_FLIGHT").is_none() {
        return Ok(());
    }

    let mut config = prost_build::Config::new();
    // Derive Serialize on the trail messages so a track can go straight to a
    // JSON API without a hand-written mirror of each protobuf type. Scoped to
    // these packages, not `.`: the live-feed messages embed
    // `prost_types::FieldMask`, which has no Serialize impl, so deriving
    // everywhere fails to compile.
    config.type_attribute(".live_trail", "#[derive(serde::Serialize)]");
    // The trail points themselves live in `common`, and only this one message
    // is reachable from a trail response — deriving on all of `common` would
    // drag in the live-feed types that embed `FieldMask`.
    config.type_attribute(".common.RadarHistoryRecord", "#[derive(serde::Serialize)]");
    config.type_attribute(".historic_trail", "#[derive(serde::Serialize)]");

    let flight_dir = Path::new("protos/flight");
    let mut flight_protos: Vec<_> = std::fs::read_dir(flight_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "proto"))
        .collect();
    // read_dir order is filesystem-dependent; a stable list keeps the build
    // reproducible and the error messages comparable between machines.
    flight_protos.sort();
    config.compile_protos(&flight_protos, &[flight_dir])?;

    Ok(())
}
