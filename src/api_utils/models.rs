use serde::Deserialize;

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
