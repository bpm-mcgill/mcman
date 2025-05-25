use reqwest::{Client};
use serde::{Deserialize};
use std::{fs::File, io};
use futures_util::StreamExt;
use std::io::Write;

// Deserialize structs for fetching version list
#[derive(Debug, Deserialize)]
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

// Deserlialize structs for fetching .jar url
#[derive(Debug, Deserialize)]
struct VersionData {
    //id: String,
    downloads: VersionDownloads
}

#[derive(Debug, Deserialize)]
struct VersionDownloads {
    server: Option<DownloadsData>
}

#[derive(Debug, Deserialize)]
struct DownloadsData {
    size: i32,
    url: String
}


#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{
    // API Url
    const VURL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json"; 

    // Create a new reqwest client to make get requests, then make a get request to the api url
    //  and parse the data into a Vec<Version> using the serde::Deserialize trait on structs and
    //  setting the .get() response type to the struct.
    let client = Client::new();
    let manifest: Manifest = client.get(VURL).send().await?.json().await?;
    let versions = manifest.versions;

    println!("Version types to fetch (R: release, S: snapshots, B: betas, A: alphas): ");
    let mut input = String::new();

    io::stdin().read_line(&mut input).expect("Error reading input fuckass");
    let first = input.trim().chars().next();
    let list_type = match first {
        Some('R') => "release",
        Some('S') => "snapshot",
        Some('B') => "old_beta",
        Some('A') => "old_alpha",
        _ => "release"
    };

    // Filter out all of the versions that aren't of the selected release type
    let selected_versions: Vec<Version> = versions.into_iter().filter(|v| v.vtype == list_type).collect();
    
    // lowk could just make dis jawn a for loop but who cares
    let sel_ids: Vec<&String> = selected_versions.iter().map(|v| &v.id).collect();
    println!("sel: {sel_ids:?}");

    println!("Select a version: ");
    input.clear();
    io::stdin().read_line(&mut input).expect("Error reading input fuckass");
    let trimd = input.trim();
    
    // 80% sure all versions after 1.3 have server .jars
    //
    // Use .find to get an Option<>, Some if the user selected version exists, and None if it
    //  doesn't. version takes the value in Some()
    if let Some(version) = selected_versions.iter().find(|v| v.id.as_str() == trimd) {

        // Fetch the selected versiondata in order to extract the .jar download url
        let data: VersionData = client.get(&version.url).send().await?.json().await?;

        // Check if the selected version has a server .jar available
        if let Some(server) = &data.downloads.server {
            println!("Downloading server .jar");
            println!("{}",server.size);
            
            // Use a stream so the entire server .jar doesn't need to be stored in memory before it
            //  is written to a file.
            let mut resp = client.get(&server.url).send().await?.bytes_stream();
            let mut file = File::create("server.jar").expect("gng");
            
            // Keep fetching more data and writing it as it is fetched until it is all downloaded
            //  and written to the server.jar file
            while let Some(chunk) = resp.next().await {
                let chunk = chunk?;
                file.write_all(&chunk).expect("balls");
            }
            println!("Download finished");
        } else {
            println!("No server jar file for that version");
        }
    } 
    Ok(())
}
