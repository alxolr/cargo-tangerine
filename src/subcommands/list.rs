use std::path::PathBuf;

use clap::Parser;

use crate::errors::Result;

#[derive(Debug, Parser)]
pub struct List {
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl List {
    pub async fn run(&self) -> Result<()> {
        println!("Listing crates in the workspace at {:?}", self.path);

        Ok(())
    }
}
