use tar::Archive;
use flate2::read::GzDecoder;
use std::error::Error;
use std::io::Read;
use autovet_core::pacman::PacmanPackage;

struct PacmanBulkUpdateRequest {
    pub docs: Vec<PacmanPackage>,
}

struct PacmanBulkGetRequest {
    pub fields: Vec<String>,
    pub limit: u32,
}

/// Synchronize package metadata from the pacman database.
pub fn sync() -> Result<(), Box<dyn Error>> {

    let client = reqwest::blocking::Client::new();
    let mut db_update = PacmenBulkUpdateRequest{docs: vec![]},

    // List current packages
    let current_packages: Vec<PacmanPackage> = client.post("http://db.autovet.fossable.org/pacman/_find").body("{\"fields\":[]}").send()?;

    // Get latest pacman database and find new packages
    for repo in vec!["core", "community", "extra", "multilib"] {
        let mut tar_gz = client.get(format!("http://mirror.fossable.org/archlinux/{repo}/os/x86_64/{repo}.db.tar.gz")).send()?;
        let mut tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        for entry in archive.entries()? {
            let mut desc = String::new();
            entry?.read_to_string(&mut desc)?;

            let package = PacmanPackage::from_desc(desc);
            if ! current_packages.iter().any(|&p| p.name == package.name && p.version == package.version) {
                db_update.docs.push(package);
            }
        }
    }

    // Post new packages
    if db_update.docs.len() > 0 {
        client.post("http://db.autovet.fossable.org/pacman/_bulk_docs").json(&db_update).send()?;
    }

    Ok(())
}