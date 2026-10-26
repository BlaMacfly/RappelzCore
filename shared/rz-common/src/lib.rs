//! Briques communes à RappelzCore (serveur) et RappelzRust (client).

pub mod config;

pub use config::Config;

/// Initialise la journalisation ; `RUST_LOG` prime sur le niveau donné.
pub fn init_logging(default_level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

/// Bannière affichée au démarrage des serveurs, comme AzerothCore.
pub fn print_banner(app: &str) {
    tracing::info!("RappelzCore {} — {app}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Émulateur Rappelz en Rust, schéma SQL AzerothCore");
}
