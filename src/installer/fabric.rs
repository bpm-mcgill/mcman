use super::{Loader,download_url,LoaderError};
use serde::Deserialize;
use reqwest::Client;
use async_trait::async_trait;
use std::{path::Path};
use tokio::{fs, process::Command, sync::OnceCell};

pub struct FabricCache {
    // Stores a list of fabric supported minecraft versions
    supported: Vec<String>,
    
    // Stores a list of all fabric loader versions
    loaders: Vec<String>,
}

pub struct FabricLoader {
    client: Client,
    
    cache: OnceCell<FabricCache>
}

impl FabricLoader {
    pub fn new(client: Client) -> Self {
        FabricLoader {
            client,
            cache: OnceCell::new()
        }
    }
    
    // Fabric is different from forge in that every version of fabric loader is available for every
    // fabric supported version of minecraft, meaning that even the oldest version of fabric can be run on
    // the latest minecraft version (if its supported by fabric). This means that there is no need to
    // store a compatibility hashmap, as every value in the hashmap would be the exact same.
    //
    // If this ever changes in the future, this function would have to change to store a
    // compatibility hashmap as forge does.
    async fn get_cache(&mut self) -> Result<&FabricCache, LoaderError> {
        self.cache.get_or_try_init(|| async {
            let resp: Vec<Versions> = self.client.get("https://meta.fabricmc.net/v2/versions/loader").send().await?.json().await?;
            let fabversions: Vec<String> = resp.iter().map(|v| v.version.clone()).collect();
        
            let resp: Vec<Versions> = self.client.get("https://meta.fabricmc.net/v2/versions/game").send().await?.json().await?;
            let mcversions: Vec<String> = resp.iter().map(|v| v.version.clone()).collect();
            Ok(FabricCache {
                supported: mcversions,
                loaders: fabversions,
            })
        }).await
    }
}

#[async_trait]
impl Loader for FabricLoader{
    async fn get_compatible_versions(&mut self, mc_version: &str) -> Result<Option<Vec<String>>, LoaderError> {
        let cache = self.get_cache().await?;
        if cache.supported.contains(&mc_version.to_string()){
            // If cloning is too slow, this can be changed to return a reference
            return Ok(Some(cache.loaders.clone()));
        }
        Ok(None)
    }
    
    async fn install_server(&self, mc_version: &str, loader_version: &str, path: &Path) -> Result<(),LoaderError>{        
        let resp: Vec<InstallrVersions> = self.client.get("https://meta.fabricmc.net/v2/versions/installer").send().await?.json().await?;
        let latest = resp.first().ok_or_else(|| LoaderError::Status("Response was empty".into()))?;
        if !path.is_dir(){
            return Err(LoaderError::NotDirectory);
        }

        let installer_path = path.join("installer.jar");
        download_url(&self.client, &latest.url, &installer_path).await?;
        
        let installer_str = installer_path.to_str().ok_or_else(|| LoaderError::PathToStr)?;
        let server_str = path.to_str().ok_or_else(|| LoaderError::PathToStr)?;
        // Command to run the downloaded installer for the specified mc version and fabric version
        // at the specified directory
        let status = Command::new("java").args(["-jar", installer_str, "server", 
            "-mcversion", mc_version,
            "-loader", loader_version,
            "-downloadMinecraft",
            "-dir", server_str]).status().await?;
        if !status.success(){
            return Err(LoaderError::Install(status));
        }

        // Clean up the installer.jar file after installation is complete
        fs::remove_file(installer_path).await?;
        Ok(())
    }
}



// Deserialize structs ----------

// This can be used for the fabric versions and minecraft versions as they are structured the same
// in the fabric meta api
#[derive(Deserialize, Debug)]
struct Versions{
    version: String,
}

// Used to fetch an installer url (although not entirely nessicary because
// the url can just be constructed via the version number using the Versions struct)
#[derive(Deserialize, Debug)]
struct InstallrVersions{
    url: String
}
