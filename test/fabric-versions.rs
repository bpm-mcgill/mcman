// Quick script to test if every version of fabric loader is supported by every minecraft version
// that can be fetched from the fabric meta api (https://meta.fabricmc.net/v2/versions/game/)
//
// If this returns false, that means something has changed with the fabric meta api that breaks how
// version compatibility works in the FabricLoader code, meaning it needs to be updated for those
// changes.
use serde::Deserialize;
use std::{collections::HashMap, io};
use reqwest::Client;

#[derive(Deserialize)]
struct Versions {
    version: String
}

#[derive(Deserialize)]
struct VersionSquared {
    loader: VersionLoad
}

#[derive(Deserialize)]
struct VersionLoad {
    version: String
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{
    let client = Client::new();
    let vs: Vec<Versions> = client.get("https://meta.fabricmc.net/v2/versions/game").send().await?.json().await?;
    let versions: Vec<String> = vs.iter().map(|v| v.version.clone()).collect();
    
    let mut compat_list: Vec<Vec<String>> = Vec::new();
    for v in versions {
        println!("Getting version: {v}");
        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{ver}", ver=v);
        let v2: Vec<VersionSquared> = client.get(url).send().await?.json().await?;
        let v2l: Vec<String> = v2.iter().map(|v2| v2.loader.version.clone()).collect();
        compat_list.push(v2l);
    }
    let first = compat_list.first().expect("empty");
    let equal = compat_list.iter().all(|c| c == first);
    println!("{equal}");
    Ok(())
}
