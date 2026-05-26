//! Build script for Rdata Station
//!
//! Automatically generates TypeScript type definitions from Rust types.

fn main() {
    // Tell Cargo to rerun this script if any of the source files change
    println!("cargo:rerun-if-changed=src/core/types.rs");
    println!("cargo:rerun-if-changed=src/commands/metadata_commands.rs");
    println!("cargo:rerun-if-changed=src/commands/metadata_cache_commands.rs");
    println!("cargo:rerun-if-changed=src/commands/cache_warming_commands.rs");

    // Export types in debug builds only
    #[cfg(debug_assertions)]
    {
        use rdata_station_lib::core::types::export_types;
        export_types();
    }
}
