use std::fs;

use clap::Parser;
use dino_server::{start_server, ProjectConfig, SwappalbeAppRouter, TenentRouter};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use crate::{build_project, CmdExector};

#[derive(Debug, Parser)]
pub struct RunOpts {
    // port to listen
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

impl CmdExector for RunOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let layer = Layer::new().with_filter(LevelFilter::INFO);
        tracing_subscriber::registry().with(layer).init();

        let filename = build_project(".")?;
        let code = fs::read_to_string(&filename)?;
        let config = ProjectConfig::load(filename.replace(".mjs", ".yml"))?;

        let routers = vec![TenentRouter::new(
            "localhost",
            SwappalbeAppRouter::try_new(code, config.routes)?,
        )];
        start_server(self.port, routers).await?;
        Ok(())
    }
}
