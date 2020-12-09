use anyhow::{Error, Result};
use pinata_sdk::{PinByFile, PinataApi};
use std::env;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    file_or_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let api_key = env::var("PINATA_API_KEY")?;
    let secret_api_key = env::var("PINATA_SECRET_API_KEY")?;

    let api = PinataApi::new(api_key, secret_api_key).unwrap();

    let result = api
        .pin_file(PinByFile::new(opt.file_or_dir))
        .await
        .map_err(Error::msg)?;

    println!("{}", result.ipfs_hash);

    Ok(())
}
