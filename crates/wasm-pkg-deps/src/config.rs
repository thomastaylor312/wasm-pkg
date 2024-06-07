//! Definitions and helpers for loading the dependency configuration file
use std::collections::BTreeMap;

use oci_distribution::{
    client::{ClientConfig, ClientProtocol},
    secrets::RegistryAuth,
    Client,
};
use serde::{Deserialize, Serialize};

/// The default config file name
pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
/// The default registry for pulling dependencies
pub const DEFAULT_REGISTRY: &str = "ghcr.io";
/// The default registry subpath for dependencies
pub const DEFAULT_REGISTRY_SUBPATH: &str = "WebAssembly";
/// The WASI package namespace
pub const WASI_PACKAGE_NAMESPACE: &str = "wasi";

/// The configuration for the dependency manager. This file indicates where to pull dependencies
/// from depending on the package namespace (e.g. the `wasi` in `wasi:http`). Besides authorization
/// details, each configuration has a registry URL to set (like `ghcr.io`) and a default subpath to
/// use for pulling dependencies. The subpath is the path before the actual artifact (e.g. if your
/// reference is ghcr.io/my/subpath/component:0.1.0, then the subpath is `my/subpath`).
///
/// Please note that packages must be named using the kebab case version of the package name (e.g.
/// `wasi-http` for `wasi:http`) and have a valid semver version (with no `v` or `V` prefix) in
/// order to be pulled from the registry properly.
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// The default package namespace to use for pulling dependencies.
    #[serde(default)]
    pub default_namespace: String,
    /// The config for the default package namespace
    #[serde(default)]
    pub default_config: RegistryConfig,
    /// A mapping of package namespaces to their configs
    #[serde(default)]
    pub namespaces: BTreeMap<String, RegistryConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_namespace: "wasi".to_string(),
            default_config: RegistryConfig::default(),
            namespaces: BTreeMap::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistryConfig {
    /// The registry to use for pulling dependencies
    pub registry: String,
    /// The protocol to use for pulling dependencies. This defaults to "https" and only accepts the
    /// strings "https" and "http". Any invalid strings will default to https instead
    pub protocol: Option<String>,
    /// The registry subpath to use for pulling dependencies. This is the path before the actual
    /// artifact (e.g. if your reference is ghcr.io/my/subpath/component:0.1.0, then the subpath
    /// would be my/subpath without any leading or trailing slashes). If no subpath is specified,
    /// this means the "root" level of the registry is used
    pub registry_subpath: Option<String>,
    /// Optional authentication details to use for the registry
    pub auth: Option<Auth>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            registry: DEFAULT_REGISTRY.to_string(),
            registry_subpath: Some(DEFAULT_REGISTRY_SUBPATH.to_string()),
            protocol: None,
            auth: None,
        }
    }
}

impl RegistryConfig {
    /// Returns an OCI client and auth for the registry
    pub fn get_client(&self) -> (Client, RegistryAuth) {
        let client = Client::new(ClientConfig {
            protocol: match self.protocol.as_deref() {
                Some("http") => ClientProtocol::Http,
                Some("https") => ClientProtocol::Https,
                Some(_) => {
                    // TODO log warning
                    ClientProtocol::Https
                }
                None => ClientProtocol::Https,
            },
            ..Default::default()
        });
        let auth = self
            .auth
            .clone()
            .map(|auth| RegistryAuth::Basic(auth.username, auth.password))
            .unwrap_or(RegistryAuth::Anonymous);
        (client, auth)
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    /// The username to use for authentication
    pub username: String,
    /// The password to use for authentication
    pub password: String,
}
