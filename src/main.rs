use std::path::PathBuf;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use oci_distribution::{
    client::{ClientConfig, ClientProtocol},
    secrets::RegistryAuth,
    Reference,
};
use oci_wasm::{WasmClient, WasmConfig};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct App {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Push a wasm component to a registry
    Push(PushArgs),
    /// Pull a wasm component from a registry
    Pull(PullArgs),
    /// Manage dependencies for a project
    #[clap(subcommand)]
    Deps(DepsSubcommand),
}

#[derive(Debug, Args)]
struct Auth {
    /// The username to use for authentication
    #[clap(
        id = "username",
        short = 'u',
        env = "WASM_PKG_USERNAME",
        requires = "password"
    )]
    pub username: Option<String>,
    /// The password to use for authentication. This is required if username is set
    #[clap(
        id = "password",
        short = 'p',
        env = "WASM_PKG_PASSWORD",
        requires = "username"
    )]
    pub password: Option<String>,
}

impl TryFrom<Auth> for RegistryAuth {
    type Error = anyhow::Error;
    fn try_from(auth: Auth) -> Result<Self, Self::Error> {
        match (auth.username, auth.password) {
            (Some(username), Some(password)) => Ok(RegistryAuth::Basic(username, password)),
            (None, None) => Ok(RegistryAuth::Anonymous),
            _ => Err(anyhow::anyhow!("Must provide both a username and password")),
        }
    }
}

#[derive(Debug, Args)]
struct Common {
    /// A comma delimited list of allowed registries to use for http instead of https
    #[clap(
        long = "insecure",
        default_value = "",
        env = "WASM_PKG_INSECURE",
        value_delimiter = ','
    )]
    pub insecure: Vec<String>,
}

#[derive(Debug, Args)]
struct PushArgs {
    #[clap(flatten)]
    pub auth: Auth,

    #[clap(flatten)]
    pub common: Common,

    /// An optional author to set for the pushed component
    #[clap(short = 'a', long = "author")]
    pub author: Option<String>,

    /// The OCI reference to push
    pub reference: Reference,

    /// The path to the file to push
    pub file: PathBuf,
}

#[derive(Debug, Args)]
struct PullArgs {
    #[clap(flatten)]
    pub auth: Auth,

    #[clap(flatten)]
    pub common: Common,

    /// The OCI reference to pull
    pub reference: Reference,

    /// The output path to write the file to
    #[clap(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum DepsSubcommand {
    /// List all dependencies of a given wasm package
    Update(DepsUpdateArgs),
}

#[derive(Debug, Args)]
struct DepsUpdateArgs {
    /// The config file to use for pulling dependencies
    #[clap(short = 'c', long = "config")]
    pub config: Option<PathBuf>,

    /// The cache directory for dependencies
    #[clap(short = 'd', long = "cache-dir")]
    pub cache_dir: Option<PathBuf>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = App::parse();
    match args.subcmd {
        SubCommand::Push(args) => handle_push(args).await,
        SubCommand::Pull(args) => handle_pull(args).await,
        SubCommand::Deps(args) => match args {
            DepsSubcommand::Update(args) => handle_deps_update(args).await,
        },
    }
}

async fn handle_push(args: PushArgs) -> anyhow::Result<()> {
    let client = get_client(args.common);
    let (conf, layer) = WasmConfig::from_component(&args.file, args.author)
        .await
        .context("Unable to parse component")?;
    let auth = args.auth.try_into()?;
    client
        .push(&args.reference, &auth, layer, conf, None)
        .await
        .context("Unable to push image")?;
    println!("Pushed {}", args.reference);
    Ok(())
}

async fn handle_pull(args: PullArgs) -> anyhow::Result<()> {
    let client = get_client(args.common);
    let auth = args.auth.try_into()?;
    let data = client
        .pull(&args.reference, &auth)
        .await
        .context("Unable to pull image")?;
    let output_path = match args.output {
        Some(output_file) => output_file,
        None => PathBuf::from(format!(
            "{}.wasm",
            args.reference.repository().replace('/', "_")
        )),
    };
    tokio::fs::write(
        &output_path,
        data.layers
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No layers found"))?
            .data,
    )
    .await
    .context("Unable to write file")?;
    println!(
        "Successfully wrote {} to {}",
        args.reference,
        output_path.display()
    );
    Ok(())
}

async fn handle_deps_update(args: DepsUpdateArgs) -> anyhow::Result<()> {
    Ok(())
}

fn get_client(common: Common) -> WasmClient {
    let client = oci_distribution::Client::new(ClientConfig {
        protocol: if common.insecure.is_empty() {
            ClientProtocol::Https
        } else {
            ClientProtocol::HttpsExcept(common.insecure)
        },
        ..Default::default()
    });

    WasmClient::new(client)
}
