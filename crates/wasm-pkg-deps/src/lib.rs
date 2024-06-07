//! A library for pulling wit dependencies from a registry
use oci_distribution::secrets::RegistryAuth;
use oci_wasm::WasmClient;

pub mod config;
pub mod manifest;

pub use config::Config;

/// A client for pulling dependencies specified in a manifest
pub struct DepsClient {
    default_client: WasmClient,
    default_auth: RegistryAuth,
    config: Config,
}

impl Default for DepsClient {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl DepsClient {
    /// Create a new `DepsClient` from the given config
    pub fn new(config: Config) -> Self {
        let (default_client, default_auth) = config.default_config.get_client();
        Self {
            default_client: WasmClient::new(default_client),
            default_auth,
            config,
        }
    }
}
