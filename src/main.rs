use clap::Parser;
mod publish;

#[derive(Debug, Parser)]
enum Subcommand {
    Publish(publish::Publish),
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

fn main() {
    let Cargo::Tangerine(opt) = Cargo::parse();

    match opt.subcmd {
        Subcommand::Publish(publish) => {
            publish.run();
        }
    }
}
