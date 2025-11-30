use clap::Args as ArgsTrait;

use crate::result::Result;

#[derive(ArgsTrait, Debug)]
pub struct Args {}

pub fn main(args: Args) -> Result<()> {
    println!("TODO: serve tasks {args:?}");
    Ok(())
}
