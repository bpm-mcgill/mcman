use std::{
    collections::HashMap,
    io,
    path::{Path},
    process::ExitStatus,
};
use reqwest::Client;
use futures_util::StreamExt;
use tempfile::{NamedTempFile};
use thiserror::Error;
use async_trait::async_trait;
use tokio::{
    fs,
    io::{AsyncWriteExt},
    sync::OnceCell
};

use serde::Deserialize;

use fabric::FabricLoader;
// use forge::ForgeLoader;
mod forge;
pub mod fabric;

// Trait implementation for modloaders to implement
#[async_trait]
pub trait Loader {
    async fn get_compatible_versions(&mut self, mc_version: &str) -> Result<Option<Vec<String>>, LoaderError>;
    async fn install_server(&self, mc_version: &str, loader_version: &str, path: &Path) -> Result<(),LoaderError>;
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum LoaderType {
    Fabric,
    //Forge,
    // Quilt,
    // NeoForge
}

pub enum ServerKind {
    Vanilla { mc: String },
    Modded {
        mc: String,
        loadertype: LoaderType,
        version: String,
    },
}

pub struct ServerInstaller {
    client: Client,
    manifest: OnceCell<Manifest>, // Stores a list of all mc versions
    loaders: HashMap<LoaderType, Box<dyn Loader>>
}

impl ServerInstaller {
    pub fn new(client: Client) -> Self {
        let mut loaders: HashMap<LoaderType, Box<dyn Loader>> = HashMap::new();
        loaders.insert(LoaderType::Fabric, Box::new(FabricLoader::new(client.clone())));
        // loaders.insert(LoaderType::Forge, Box::new(ForgeLoader::new(client.clone())));
        ServerInstaller { 
            client,
            manifest: OnceCell::new(),
            loaders,
        }
    }
    
    pub async fn get_versions(&mut self) -> Result<Vec<String>, InstallerError> {
        Ok(self.get_manifest().await?.versions.iter().map(|v| v.id.clone()).collect())
    }

    pub async fn install_server(&mut self, server_info: ServerKind, path: &Path) -> Result<(), InstallerError>{
        match server_info {
            ServerKind::Vanilla {mc} => {
                let mani = self.get_manifest().await?;
                let version_url = mani.versions.iter()
                    .find(|v| v.id.as_str() == mc)
                    .map(|v| v.url.clone());
                let Some(url) = version_url else {
                    return Err(InstallerError::InvalidVersion);
                };

                let version: VersionManifest = self.client.get(&url).send().await?.json().await?;
                let Some(jar) = version.downloads.server else {
                    return Err(InstallerError::NoJarFile(mc));
                };

                let jar_path = path.join("server.jar");
                download_url(&self.client, &jar.url, jar_path).await?;
                
            }
            ServerKind::Modded {mc, loadertype, version} => {
                let loader = self.loaders.get_mut(&loadertype)
                    .ok_or_else(|| InstallerError::InvalidLoader(loadertype))?;
                let compat = loader.get_compatible_versions(&mc).await?
                    .ok_or_else(|| InstallerError::NotCompatible(version.clone(), mc.clone()))?;
                
                if !compat.contains(&version){
                    return Err(InstallerError::NotCompatible(version, mc));
                }
                
                loader.install_server(&mc, &version, path).await?;
            }
        };
        Ok(())
    }

    pub async fn get_loader_compat(&mut self, mc_version: &str, loadertype: LoaderType) -> Result<Option<Vec<String>>, InstallerError> {
        let loader = self.loaders.get_mut(&loadertype).ok_or(InstallerError::InvalidLoader(loadertype))?;
        Ok(loader.get_compatible_versions(mc_version).await?)
    }

    // Method to fetch manifest. Every method that needs the manifest will call this instead of
    // accessing it directly to ensure the manifest is only ever fetched once during runtime
    async fn get_manifest(&mut self) -> Result<&Manifest, InstallerError>{
        self.manifest
            .get_or_try_init(|| async {
                let manifest: Manifest = self.client.get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json").send().await?.json().await?;
                Ok(manifest)
            }).await
    }
}

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("The loader provided")]
    InvalidLoader(LoaderType),

    #[error(transparent)]
    Loader(#[from] LoaderError),

    #[error("The loader version ({0}) isn't compatible with ({1})")]
    NotCompatible(String,String),

    #[error("The mc version provided is not a valid minecraft version")]
    InvalidVersion,

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    
    #[error(transparent)]
    Download(#[from] DownloadError),

    #[error("No vanilla jar file found for {0}")]
    NoJarFile(String),
}

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("The provided path isn't a directory")]
    NotDirectory,

    // Could potentially make this also take a String, so this can report the exact path that
    // failed to convert
    #[error("Couldn't convert path to string")]
    PathToStr,

    #[error("Status check failed: '{0}'")]
    Status(String),
    
    #[error("Installation command execution failed: '{0}'")]
    Install(ExitStatus),
    
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Download(#[from] DownloadError),
    
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("HTTP {0} when fetching {1}")]
    Http(reqwest::StatusCode, String),
    
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    
    #[error(transparent)]
    Io(#[from] io::Error),
    
    #[error("Failed to persist temp file: {0}")]
    Persist(#[from] tempfile::PathPersistError),
}

pub async fn download_url<P>(client: &Client, url: &str, path: P) -> Result<(), DownloadError> 
where P: AsRef<Path>
{
    let destination = path.as_ref();
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).await?;
    }
    let resp = client.get(url).send().await?;
    let status = resp.status();
    if !status.is_success(){
        return Err(DownloadError::Http(status, url.to_string()));
    }

    let tmp = {
        let dir = destination.parent().unwrap_or_else(|| Path::new("."));
        NamedTempFile::new_in(dir)?.into_temp_path()
    };
    let mut file = fs::File::create(&tmp).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    tmp.persist(destination)?;
    Ok(())
}

// Default impl for compiler
impl Default for ServerInstaller {
    fn default() -> Self {
        // TODO: fix this shit
        Self::new(Client::new())
    }
}


// Deserialize structs for fetching version list
#[derive(Debug, Deserialize)]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub vtype: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub versions: Vec<Version>
}

// Deserlialize structs for fetching .jar url
#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    //id: String,
    pub downloads: VersionDownloads
}

#[derive(Debug, Deserialize)]
pub struct VersionDownloads {
    pub server: Option<DownloadsData>
}

#[derive(Debug, Deserialize)]
pub struct DownloadsData {
    pub size: u64,
    pub url: String
}
