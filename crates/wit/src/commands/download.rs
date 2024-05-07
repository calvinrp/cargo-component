use crate::{config::Config, download_wit_package, DownloadOptions};
use anyhow::{Context, Result};
use cargo_component_core::{command::CommonOptions, registry::find_url};
use clap::Args;
use semver::VersionReq;
use std::path::PathBuf;
use warg_protocol::registry::PackageName;

/// Download a WIT package from a registry.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct DownloadCommand {
    /// The common command options.
    #[clap(flatten)]
    pub common: CommonOptions,

    /// Use the specified registry name for the package.
    #[clap(long = "registry", value_name = "REGISTRY")]
    pub registry: Option<String>,

    /// The package name to download.
    /// TODO change to include version syntax
    #[clap(value_name = "NAME")]
    pub package: PackageName,

    /// The version requirement of the package to download; defaults to `*`.
    #[clap(long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Skip local cache and check registry for updates.
    #[clap(long)]
    pub update: bool,

    /// The path to write the WIT text file.
    ///
    /// If not specified, the output will be written as the `{package_name}.wit`.
    #[clap(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// The path to initialize the package in.
    #[clap(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,
}

impl DownloadCommand {
    /// Executes the command.
    pub async fn exec(self) -> Result<()> {
        log::debug!("executing download command");

        let terminal = self.common.new_terminal();

        let warg_config = warg_client::Config::from_default_file()?.unwrap_or_default();

        // config file optional
        let config = Config::from_default_file()?;
        let url = if let Some((config, _)) = &config {
            find_url(
                self.registry.as_deref(),
                &config.registries,
                warg_config.home_url.as_deref(),
            )?
        } else {
            warg_config
                .home_url
                .as_deref()
                .context("warg home URL not configured")?
        };

        // if user specifies exact verion, then set the `VersionReq` to exact match
        let version = match &self.version {
            Some(version) => VersionReq::parse(&format!("={}", version))?,
            None => VersionReq::STAR,
        };

        download_wit_package(
            DownloadOptions {
                warg_config: &warg_config,
                url,
                package: &self.package,
                version: &version,
                output: self.output.as_ref(),
                update: self.update,
            },
            &terminal,
        )
        .await
    }
}
