use mcman::api_utils::api_handler::ApiHandler;
use std::io;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{

    let mut handler = ApiHandler::new();

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
    
    let version_list = handler.get_versions(list_type).await?;
    println!("{version_list:?}");

    println!("Select a version: ");
    input.clear();
    io::stdin().read_line(&mut input).expect("Error reading input fuckass");
    let trimd = input.trim();

    // 80% sure all versions after 1.2.5 have server .jars
    handler.download_version(trimd).await?;
    
    Ok(())
}
