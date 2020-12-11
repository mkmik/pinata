use anyhow::{Error, Result};
use pinata_sdk::{PinByFile, PinOptions, PinataApi};
use std::env;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    file_or_dir: String,
    #[structopt(short = "1", help = "CIDv1, base32")]
    v1: bool,
    //    #[structopt(long, help = "Name of the pin")]
    //    name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let api_key = env::var("PINATA_API_KEY")?;
    let secret_api_key = env::var("PINATA_SECRET_API_KEY")?;

    let api = PinataApi::new(api_key, secret_api_key).unwrap();
    let mut req = PinByFile::new(opt.file_or_dir);
    req = req.set_options(PinOptions {
        host_nodes: None,
        custom_pin_policy: None,
        cid_version: Some(if opt.v1 { 1 } else { 0 }),
    });
    let result = api.pin_file(req).await.map_err(Error::msg)?;

    println!("{}", result.ipfs_hash);

    Ok(())
}
