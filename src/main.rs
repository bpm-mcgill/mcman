use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::io;

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    id: String,
    #[serde(rename = "type")]
    vtype: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    versions: Vec<Version>
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{
    const VURL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json"; 
    let client = Client::new();
    let manifest: Manifest = client.get(VURL).send().await?.json().await?;
    let versions = manifest.versions;

    println!("Version types to fetch (R: release, S: snapshots, B: betas, A: alphas): ");
    let mut input = String::new();
    //if let Err(e) = io::stdin().read_line(&mut input) {
    //    eprintln!("couldn't read input");
    //}
    // or 

    let _ = io::stdin().read_line(&mut input).expect("Error reading input fuckass");
    let first = input.trim().chars().next();
    let list_type = match first {
        Some('R') => String::from("release"),
        Some('S') => String::from("snapshot"),
        Some('B') => String::from("old_beta"),
        Some('A') => String::from("old_alpha"),
        _ => String::from("release")
    };

    let selected_versions: Vec<Version> = versions.into_iter().filter(|v| v.vtype == list_type).collect();
    
    for v in selected_versions {
        let v_num = v.id;
        println!("{v_num}");
    }


    //println!("sel_vers = {selected_versions:#?}");
    
    //let latest = versions.first();
    //println!("latest = {latest:#?}");
    Ok(())
}
