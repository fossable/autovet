use autovet_core::package::Package;
use flate2::read::GzDecoder;
use std::error::Error;
use std::io::Read;
use tar::Archive;

/// Synchronize package metadata from the pacman database.
pub fn sync() -> Result<(), Box<dyn Error>> {
	// List current packages
	let current_packages: Vec<Package> = Package::list("pacman")?;

	// Get latest pacman database and find new packages
	for repo in vec!["core", "community", "extra", "multilib"] {
		let tar_gz = reqwest::blocking::get(format!(
			"http://mirror.fossable.org/archlinux/{repo}/os/x86_64/{repo}.db.tar.gz"
		))?;
		let tar = GzDecoder::new(tar_gz);
		let mut archive = Archive::new(tar);

		for entry in archive.entries()? {
			let mut desc = String::new();
			entry?.read_to_string(&mut desc)?;

			let mut package = Package::from_pacman_desc(desc);
			if !current_packages
				.iter()
				.any(|p| p.name == package.name && p.version == package.version)
			{
				package.update()?;
			}
		}
	}

	Ok(())
}
