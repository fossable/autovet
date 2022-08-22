use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use simple_error::bail;
use std::default::Default;
use std::error::Error;

#[derive(Serialize, Deserialize, Default)]
pub enum PackageChannel {
	#[default]
	None,
	Pacman,
	Debian,
	PyPi,
	MavenCentral,
	Npm,
	CratesIo,
	DockerHub,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Package {
	pub _id: String,

	pub _rev: String,

	pub channel: PackageChannel,

	pub name: String,

	pub version: String,

	pub filename: Option<String>,

	pub description: Option<String>,

	pub size: Option<String>,

	pub md5sum: Option<String>,

	pub sha256sum: Option<String>,

	pub upstream_url: Option<String>,

	pub license: Option<String>,

	pub arch: Option<String>,

	pub build_date: Option<String>,
}

mod db {
	use serde::Deserialize;

	#[derive(Deserialize)]
	pub struct UpdateDocumentResponse {
		pub id: String,
		pub ok: String,
		pub rev: String,
	}
}

impl Package {
	pub fn from_pacman_desc(desc: String) -> Package {
		let lines: Vec<&str> = desc.split("\n").collect();
		let mut package = Package::default();
		package.channel = PackageChannel::Pacman;

		for i in 0..lines.len() {
			match lines[i] {
				"%NAME%" => package.name = lines[i + 1].to_string(),
				"%FILENAME%" => package.filename = Some(lines[i + 1].to_string()),
				"%VERSION%" => package.version = lines[i + 1].to_string(),
				"%DESC%" => package.description = Some(lines[i + 1].to_string()),
				"%CSIZE%" => package.size = Some(lines[i + 1].to_string()),
				"%MD5SUM%" => package.md5sum = Some(lines[i + 1].to_string()),
				"%SHA256SUM%" => package.sha256sum = Some(lines[i + 1].to_string()),
				"%URL%" => package.upstream_url = Some(lines[i + 1].to_string()),
				"%LICENSE%" => package.license = Some(lines[i + 1].to_string()),
				"%ARCH%" => package.arch = Some(lines[i + 1].to_string()),
				"%BUILDDATE%" => package.build_date = Some(lines[i + 1].to_string()),
				_ => continue,
			}
		}

		package
	}

	pub fn list(channel: &str) -> Result<Vec<Package>, Box<dyn Error>> {
		todo!()
	}

	pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
		let rs = reqwest::blocking::Client::new()
			.post(format!(
				"http://db.autovet.fossable.org/packages/{}?ref={}",
				self._id, self._rev
			))
			.json(&self)
			.send()?;

		match rs.status() {
			StatusCode::CREATED | StatusCode::ACCEPTED => {
				let rs: db::UpdateDocumentResponse = rs.json()?;
				self._rev = rs.rev;
				Ok(())
			}
			_ => bail!(""),
		}
	}
}
