use std::path::PathBuf;

use clap::Parser;

use crate::errors::Result;

#[derive(Debug, Parser)]
pub struct Publish {
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl Publish {
    pub async fn run(&self) -> Result<()> {
        println!("Publishing from {:?}", self.path);

        Ok(())
    }
}
