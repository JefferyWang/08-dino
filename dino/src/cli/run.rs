use std::{fs, path::Path, time::Duration};

use clap::Parser;
use dino_server::{start_server, ProjectConfig, SwappalbeAppRouter, TenentRouter};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use tokio::sync::mpsc::channel;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use crate::{build_project, CmdExector};

const MONITOR_FS_INTERVAL: Duration = Duration::from_secs(2);

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

        let router = SwappalbeAppRouter::try_new(code, config.routes)?;
        let routers = vec![TenentRouter::new("localhost", router.clone())];

        tokio::spawn(async_watch(".", router));

        start_server(self.port, routers).await?;
        Ok(())
    }
}

fn get_code_and_config() -> anyhow::Result<(String, ProjectConfig)> {
    let filename = build_project(".")?;
    let config = filename.replace(".mjs", ".yml");
    let code = fs::read_to_string(filename)?;
    let config = ProjectConfig::load(config)?;
    Ok((code, config))
}

async fn async_watch(p: impl AsRef<Path>, router: SwappalbeAppRouter) -> anyhow::Result<()> {
    let (tx, rx) = channel(1);

    let mut debouncer = new_debouncer(MONITOR_FS_INTERVAL, move |res: DebounceEventResult| {
        tx.blocking_send(res).unwrap();
    })?;

    debouncer
        .watcher()
        .watch(p.as_ref(), notify::RecursiveMode::Recursive)?;

    let mut stream = ReceiverStream::new(rx);
    while let Some(ret) = stream.next().await {
        match ret {
            Ok(events) => {
                let mut need_swap = false;
                // config.yml change, or any ".ts" / ".js" file change
                for event in events {
                    let path = event.path;
                    let ext = path.extension().unwrap_or_default();
                    if path.ends_with("config.yml") || ext == "ts" || ext == "js" {
                        info!("File changed: {}", path.display());
                        need_swap = true;
                        break;
                    }
                }

                if need_swap {
                    let (code, config) = get_code_and_config()?;
                    router.swap(code, config.routes)?;
                }
            }
            Err(e) => {
                warn!("Error: {:?}", e);
            }
        }
    }
    Ok(())
}
