//! Cache Command Handler
//!
//! ## Responsibility
//! CLI entry point for cache management commands.
//!
//! ## BSP-146: Artifact Cache
//! - `primer cache status` - Show cache statistics
//! - `primer cache purge` - Clear all cached data

use primer::ports::ArtifactCache;
use tracing::info;

/// Show cache statistics.
pub async fn status() -> anyhow::Result<()> {
    let cache = primer::adapters::cache::CacheManager::new()?;
    let stats = cache.stats()?;
    println!("{}", stats.summary());
    Ok(())
}

/// Purge all cached data.
pub async fn purge(force: bool) -> anyhow::Result<()> {
    let cache = primer::adapters::cache::CacheManager::new()?;

    if !force {
        println!("This will delete all cached Helm charts and rendered manifests.");
        println!("Cache directory: {}", cache.root().display());
        print!("Continue? [y/N] ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let stats = cache.purge()?;
    info!(
        "Purged {} rendered manifests, {} helm charts",
        stats.rendered_purged, stats.charts_purged
    );
    println!(
        "Purged {} rendered manifests, {} helm charts",
        stats.rendered_purged, stats.charts_purged
    );
    Ok(())
}
