use autovet_core::package::Package;
use autovet_core::worker::Worker;
use log::info;
use serde_json::json;
use std::{error::Error, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
	env_logger::init_from_env(env_logger::Env::new());

	// Register worker
	let worker = Worker {
		_id: String::from("123"),
		hostname: gethostname::gethostname().to_string_lossy().to_string(),
		last_checkin: 0,
	};

	loop {
		// Attempt to first select a package that another worker has dropped
		// TODO

		// Select a package that has never been processed
		let mut packages: Vec<Package> = Package::find(json!(
		{
			"selector": {
				"channel": "Pacman",
				"worker": {
					"$exists": false
				}
			},
			"limit": 1
		}))?;

		if packages.len() == 0 {
			// Wait for a package to process
			info!("Waiting for a package to process");
			std::thread::sleep(Duration::from_secs(100));
			continue;
		}

		let package = &mut packages[0];

		// Lock onto this package
		package.worker = Some(worker._id.clone());
		if package.update().is_err() {
			// There may have been a collision, so just try again
			continue;
		} else {
			info!("Locked onto package: {}", package.name);
		}

		// Run static analysis
		// TODO

		// Run dynamic analysis
		// TODO
	}
}
