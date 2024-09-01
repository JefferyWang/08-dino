mod build;
mod init;
mod run;

pub use build::BuildOpts;
use clap::Parser;
use enum_dispatch::enum_dispatch;
pub use init::InitOpts;
pub use run::RunOpts;

#[derive(Debug, Parser)]
#[command(name = "dino", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum SubCommand {
    #[command(name = "init", about = "Initialize a new Dino project")]
    Init(InitOpts),
    #[command(name = "build", about = "Build a Dino project")]
    Build(BuildOpts),
    #[command(name = "run", about = "Run user's dino project")]
    Run(RunOpts),
}
