//! Bundles Topcoat's browser assets at startup.
//!
//! Topcoat apps usually ship the bundle that `topcoat asset bundle` writes next
//! to the executable. Running the same bundler on our own executable keeps
//! typdeck installable with plain `cargo install`.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};

use topcoat::asset::AssetBundle;
use topcoat_asset::{Bundler, BundlerConfig};

pub fn bundle() -> Result<AssetBundle, String> {
    let exe =
        std::env::current_exe().map_err(|error| format!("failed to locate typdeck: {error}"))?;
    let binary =
        fs::read(&exe).map_err(|error| format!("failed to read {}: {error}", exe.display()))?;

    let cache = dirs::cache_dir()
        .ok_or("failed to locate the cache directory")?
        .join("typdeck");
    // A bundle belongs to exactly one build, so each build gets its own.
    let mut hasher = DefaultHasher::new();
    binary.hash(&mut hasher);
    let dir = cache.join(format!("assets-{:016x}", hasher.finish()));

    Bundler::new(&BundlerConfig::new().cache_dir(cache.join("downloads")))
        .bundle(&binary, &dir)
        .map_err(|error| format!("failed to bundle assets: {error}"))?;
    AssetBundle::load_dir(&dir).map_err(|error| format!("failed to load assets: {error}"))
}
