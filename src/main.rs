use clap::Parser;
use errors::Result;
use subcommands::publish::Publish;

mod errors;
mod subcommands;

#[derive(Debug, Parser)]
enum Subcommand {
    Publish(Publish),
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
    }

    Ok(())
}
