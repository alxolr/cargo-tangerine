use clap::Parser;
use errors::Result;
use subcommands::{list::List, publish::Publish};

mod errors;
mod models;
mod subcommands;
mod utils;

#[derive(Debug, Parser)]
enum Subcommand {
    Publish(Publish),
    List(List),
}

#[derive(Debug, Parser)]
pub struct Opt {
    #[clap(subcommand)]
    subcmd: Subcommand,
}

#[derive(Debug, Parser)]
#[clap(name = "cargo-tangerine", bin_name = "cargo", version)]
enum Cargo {
    Tangerine(Opt),
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cargo::Tangerine(opt) = Cargo::parse();

    match opt.subcmd {
        Subcommand::Publish(publish) => {
            publish.run().await?;
        }
        Subcommand::List(list) => {
            list.run().await?;
        }
    }

    Ok(())
}
