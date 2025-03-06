use clap::Parser;

#[derive(Debug, Parser)]
pub struct Publish {}

impl Publish {
    pub fn run(&self) {
        println!("Publishing as is");
    }
}
