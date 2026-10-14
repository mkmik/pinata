use anyhow::{bail, Context, Result};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use structopt::StructOpt;
use walkdir::WalkDir;

const PIN_FILE_URL: &str = "https://api.pinata.cloud/pinning/pinFileToIPFS";

#[derive(StructOpt, Debug)]
struct Opt {
    file_or_dir: String,
    #[structopt(short = "1", help = "CIDv1, base32")]
    v1: bool,
    //    #[structopt(long, help = "Name of the pin")]
    //    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PinnedObject {
    ipfs_hash: String,
}

/// Builds the multipart form the same way pinata-sdk did: a single file is sent under its
/// file name, a directory is walked and each file is sent as "<dirname>/<relative path>".
fn file_form(file_or_dir: &str) -> Result<Form> {
    let base = Path::new(file_or_dir);
    let base_name = base
        .file_name()
        .and_then(|n| n.to_str())
        .context("path has no valid file name")?;

    let mut form = Form::new();
    if base.is_dir() {
        for entry in WalkDir::new(base) {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let rel = path.strip_prefix(base)?;
            let rel = rel.to_str().context("non UTF-8 path")?;
            let part = Part::bytes(fs::read(path)?).file_name(format!("{base_name}/{rel}"));
            form = form.part("file", part);
        }
    } else {
        let part = Part::bytes(fs::read(base)?).file_name(base_name.to_string());
        form = form.part("file", part);
    }
    Ok(form)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let api_key = env::var("PINATA_API_KEY")?;
    let secret_api_key = env::var("PINATA_SECRET_API_KEY")?;

    let cid_version = if opt.v1 { 1 } else { 0 };
    let form = file_form(&opt.file_or_dir)?
        .text("pinataOptions", format!(r#"{{"cidVersion":{cid_version}}}"#));

    let res = reqwest::Client::new()
        .post(PIN_FILE_URL)
        .header("pinata_api_key", api_key)
        .header("pinata_secret_api_key", secret_api_key)
        .multipart(form)
        .send()
        .await?;
    if !res.status().is_success() {
        bail!("pinata: {}: {}", res.status(), res.text().await?);
    }
    let result: PinnedObject = res.json().await?;

    println!("{}", result.ipfs_hash);

    Ok(())
}
