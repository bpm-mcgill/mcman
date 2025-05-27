use std::{fs::File, io::{self, Write}};
use reqwest::Client;
use futures_util::StreamExt;

use super::urls;
use super::models::{
    Manifest,
    VersionManifest
};

pub struct ApiHandler {
    client: Client,
    cached_manifest: Option<Manifest>
}

impl ApiHandler {
    pub fn new() -> Self {
        ApiHandler {
            client: Client::new(),
            cached_manifest: None,
        }
    }

    // Updates the manifest cache and returns whether the operation succeeded or failed
    pub async fn cache_manifest(&mut self) -> Result<(), reqwest::Error>{
        let manifest: Manifest = self.client.get(urls::MOJANG).send().await?.json().await?;
        self.cached_manifest = Some(manifest);
        Ok(())
    }  

    // Maybe could refactor using let else (?)
    // Returns the cached manifest
    // If the manifest isn't cached, this method will fetch and cache the manifest, then return the
    //  manifest.
    async fn get_manifest(&mut self) -> Result<&Manifest, reqwest::Error>{
        // If the manifest cache is empty, fetch the manifest for the user rather than throw an
        //  error
        if self.cached_manifest.is_none() {
            self.cache_manifest().await?;
        }
        
        // .expect is okay since manifest was verified to be of Some() type.
        //  if the call to update the manifest cache failed and was of None() type,
        //  the code would have already returned a reqwest::Error
        let manifest: &Manifest = self.cached_manifest.as_ref().expect("Somehow manifest is empty here 🤷‍♂️");
        Ok(manifest)
    }
    
    // Return a vector of the version id strings that are of the selected version type
    pub async fn get_versions(&mut self, version_type: &str) -> Result<Vec<String>, reqwest::Error>{
        let manifest: &Manifest = self.get_manifest().await?;

        let selected_versions: Vec<String> = manifest.versions
            .iter()
            .filter(|v| v.vtype == version_type) // Filter out all versions that aren't of the version_type
            .map(|v| v.id.clone())               // Get a owned clone of the version id
            .collect();                          // Convert from iter to vec
        Ok(selected_versions)
    }

    pub async fn download_version(&mut self, version_id: &str) -> Result<(),reqwest::Error>{
        let manifest: &Manifest = self.get_manifest().await?;
        
        // The url needs to be extracted and cloned so it can be an owned value, meaning that the
        //  reference to manifest (which keeps a mutable reference to self in scope as long as it
        //  is in scope) can be dropped early, allowing for a immutable reference self to be used
        //  when calling self.client
        let version = manifest.versions
            .iter()
            .find(|v| v.id.as_str() == version_id)
            .map(|v| v.url.clone());
        
        // Technically not needed here as manifest isn't used after this point, but imma leave dis jawn anyways
        drop(manifest);
       
        // If the version doesn't exist, exit and return an Error
        // Could use ok_or once I implement a custom error type to encapsulate reqwest::Errors and
        //  custom errors
        //
        //  Implement From trait on custom error type: (rustbook excerpt)
        // If we also define impl From<io::Error> for OurError to construct an instance of OurError from an io::Error,
        //  then the ? operator calls in the body of read_username_from_file will call from and convert the error types 
        //  without needing to add any more code to the function.
        //
        //let version = version.ok_or()
        let Some(v) = version else {
            println!("Version doesn't exist");
            return Ok(());
        };

        // Fetch the version's manifest in order to extract the .jar download url
        let data: VersionManifest = self.client.get(&v).send().await?.json().await?;
        
        let Some(server) = &data.downloads.server else {
            println!("No server.jar exists for this version");
            return Ok(());
        };
        println!("Downloading server .jar (size: {} KB)", server.size/1000);
        // Use a stream so the entire server .jar doesn't need to be stored in memory before it
        //  is written to a file.
        let mut resp = self.client.get(&server.url).send().await?.bytes_stream();
        let mut file = File::create("server.jar").expect("gng");
        
        // Keep fetching more data and writing it as it is fetched until it is all downloaded
        //  and written to the server.jar file
        while let Some(chunk) = resp.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).expect("balls");
        }
        println!("Download finished");
        Ok(())
    }
    
}

// Impl Default to make the compiler happy
impl Default for ApiHandler {
    fn default() -> Self {
        Self::new()
    }
}
