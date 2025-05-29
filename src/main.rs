use mcman::installer::{LoaderType, ServerInstaller, ServerKind};
use std::path::Path;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let path = "/home/user/Documents/install_test/";
    let client = Client::new();
    let mut installer = ServerInstaller::new(client);

    let versions = installer.get_versions().await?;
    let latest_mc = versions.first().unwrap();
    let compat = installer.get_loader_compat(latest_mc, LoaderType::Fabric).await?.unwrap();
    let latest_fab = compat.first().unwrap();
    
    // let serv = ServerKind::Modded { mc: latest_mc.into(), loadertype: LoaderType::Fabric, version: latest_fab.into() };
    let serv = ServerKind::Vanilla { mc: latest_mc.into() };
    installer.install_server(serv, Path::new(path)).await?; 
    

    // println!("Version types to fetch (R: release, S: snapshots, B: betas, A: alphas): ");
    // let mut input = String::new();

    // io::stdin().read_line(&mut input).expect("Error reading input fuckass");
    // let first = input.trim().chars().next();
    // let list_type = match first {
        // Some('R') => "release",
        // Some('S') => "snapshot",
        // Some('B') => "old_beta",
        // Some('A') => "old_alpha",
        // _ => "release"
    // };
    
    // let version_list = handler.get_versions(list_type).await?;
    // println!("{version_list:?}");

    // println!("Select a version: ");
    // input.clear();
    // io::stdin().read_line(&mut input).expect("Error reading input fuckass");
    // let trimd = input.trim();
    
    // just throw an error if wrong version for testing
    // version will also hold the same thing as trimd but who care lowk
    // let version = version_list.iter().find(|v| *v == trimd).expect("Version not found");

    // 80% sure all versions after 1.2.5 have server .jars
    //handler.download_version(trimd).await?;
    
    // println!("Pick a fabric loader version:");
    // input.clear();
    // io::stdin().read_line(&mut input).expect("Error reading input fuckass");
    // let trimd = input.trim();

    Ok(())
}
