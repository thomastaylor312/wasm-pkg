//! Definitions and helpers for loading the `wit.toml` and `wit.lock` dependency manifest files
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The dependency configuration format. The key names are the package name you want to pull (e.g.
/// `wasi:http`).
///
/// Please note that packages in the registry you are pulling from must be named using the kebab
/// case version of the package name (e.g. `wasi-http` for `wasi:http`) by default and have a valid
/// semver version (with no `v` or `V` prefix) in order to be pulled from the registry properly. You
/// can override the package name in the manifest as desired
pub type WitManifest = BTreeMap<String, ManifestEntry>;

/// A single entry in the dependency manifest
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ManifestEntry {
    /// Dependency version, without any additional config
    Version(String),
    /// A dependency with additional config
    Config(DependencyConfig),
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DependencyConfig {
    /// The version of the dependency
    pub version: String,
    /// The registry to use for pulling dependencies
    pub registry: Option<String>,
    /// The protocol to use for pulling dependencies. This defaults to "https" and only accepts the
    /// strings "https" and "http". Any invalid strings will default to https instead
    pub protocol: Option<String>,
    /// The registry subpath to use for pulling dependencies. This is the path before the actual
    /// artifact (e.g. if your reference is ghcr.io/my/subpath/component:0.1.0, then the subpath
    /// would be my/subpath without any leading or trailing slashes). If no subpath is specified,
    /// this means the "root" level of the registry is used
    pub registry_subpath: Option<String>,
    /// An override of the package name to use when pulling the dependency from the registry. This
    /// is useful if you want to use a different package name that doesn't match the kebab case form
    /// of the package name. This will be appended to the registry subpath to form the full package
    /// name (e.g. if your registry subpath is `my/subpath` and the package name is `my-wasi-http`,
    /// then the full package name would be `my/subpath/my-wasi-http`)
    pub package_name: Option<String>,
}
