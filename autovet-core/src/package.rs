use crate::analysis::Analysis;
use log::info;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use simple_error::bail;
use std::collections::HashMap;
use std::default::Default;
use std::error::Error;

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
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

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct Package {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub _id: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub _rev: Option<String>,

	pub channel: PackageChannel,

	pub name: String,

	pub version: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub filename: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub size: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub md5sum: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub sha256sum: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub upstream_url: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub arch: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub build_date: Option<String>,

	/// The analysis results
	#[serde(skip_serializing_if = "Option::is_none")]
	pub analysis: Option<Vec<Analysis>>,

	/// The ID of the worker assigned to process this package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub worker: Option<String>,
}

mod db {
	use serde::Deserialize;

	#[derive(Deserialize)]
	pub struct UpdateDocumentResponse {
		pub id: String,
		pub ok: bool,
		pub rev: String,
	}

	#[derive(Deserialize)]
	pub struct FindPackageResponse {
		pub docs: Vec<super::Package>,
	}
}

impl Package {
	pub fn from_pacman_desc(desc: &str) -> Result<Package, Box<dyn Error>> {
		let lines: Vec<&str> = desc.split("\n").collect();
		let mut package = Package::default();
		package.channel = PackageChannel::Pacman;

		for i in 0..lines.len() {
			match lines[i].trim() {
				"%NAME%" => package.name = lines[i + 1].trim().to_string(),
				"%FILENAME%" => package.filename = Some(lines[i + 1].trim().to_string()),
				"%VERSION%" => package.version = lines[i + 1].trim().to_string(),
				"%DESC%" => package.description = Some(lines[i + 1].trim().to_string()),
				"%CSIZE%" => package.size = Some(lines[i + 1].trim().to_string()),
				"%MD5SUM%" => package.md5sum = Some(lines[i + 1].trim().to_string()),
				"%SHA256SUM%" => package.sha256sum = Some(lines[i + 1].trim().to_string()),
				"%URL%" => package.upstream_url = Some(lines[i + 1].trim().to_string()),
				"%LICENSE%" => package.license = Some(lines[i + 1].trim().to_string()),
				"%ARCH%" => package.arch = Some(lines[i + 1].trim().to_string()),
				"%BUILDDATE%" => package.build_date = Some(lines[i + 1].trim().to_string()),
				_ => continue,
			}
		}

		if package.name == "" {
			bail!("No package name found");
		}

		if package.version == "" {
			bail!("No package version found");
		}

		Ok(package)
	}

	pub fn find(body: serde_json::Value) -> Result<Vec<Package>, Box<dyn Error>> {
		let rs = reqwest::blocking::Client::new()
			.post("http://db.auto.vet:5984/packages/_find")
			.header(reqwest::header::AUTHORIZATION, "Basic YWRtaW46YWRtaW4=")
			.header(reqwest::header::ACCEPT, "application/json")
			.header(reqwest::header::CONTENT_TYPE, "application/json")
			.body(serde_json::to_string(&body)?)
			.send()?;

		match rs.status() {
			StatusCode::OK => {
				let rs: db::FindPackageResponse = rs.json()?;
				Ok(rs.docs)
			}
			_ => bail!(""),
		}
	}

	pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
		let rs = if self._id.is_none() {
			info!("Creating new package: {}", self.name);

			reqwest::blocking::Client::new()
				.post("http://db.auto.vet:5984/packages")
				.header(reqwest::header::AUTHORIZATION, "Basic YWRtaW46YWRtaW4=")
				.header(reqwest::header::ACCEPT, "application/json")
				.json(&self)
				.send()?
		} else {
			info!("Updating package: {}", self.name);
			reqwest::blocking::Client::new()
				.put(format!(
					"http://db.auto.vet:5984/packages/{}?rev={}",
					self._id.as_ref().unwrap(),
					self._rev.as_ref().unwrap()
				))
				.header(reqwest::header::AUTHORIZATION, "Basic YWRtaW46YWRtaW4=")
				.json(&self)
				.send()?
		};

		match rs.status() {
			StatusCode::CREATED | StatusCode::ACCEPTED => {
				let rs: db::UpdateDocumentResponse = rs.json()?;
				self._rev = Some(rs.rev);
				self._id = Some(rs.id);
				Ok(())
			}
			_ => bail!(""),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_parse_pacman_desc() {
		assert_eq!(
			Package::from_pacman_desc(
				r#"%FILENAME%
				grep-3.7-1-x86_64.pkg.tar.zst
				
				%NAME%
				grep
				
				%BASE%
				grep
				
				%VERSION%
				3.7-1
				
				%DESC%
				A string search utility
				
				%GROUPS%
				base-devel
				
				%CSIZE%
				254020
				
				%ISIZE%
				846791
				
				%MD5SUM%
				df80335ae92442feb3c9bfdb9d91eff6
				
				%SHA256SUM%
				eca12ceaef774299ed7022de9b00dd7ce379186da4c223cf357fc157a9064da6
				
				%PGPSIG%
				iQIrBAABCgAdFiEEVyJOyaX8pvvJK7iqShr8NF6+GPgFAmEe3IIACgkQShr8NF6+GPh9jg++KqhTxYryohCP8quWh9CUrrsLmGBipBTvEzAs/hOs3JsaqezSx8SETMvFYtAvrX0ACap9T2rFsfudrRF4Rd7aHw4OCBk2klHKeQK/2z4JjQ7unJh0Q59fAkrBFf1T0aCP9gCJr6FC08+UACihhZhqTxO5dvsD1SAdTqf61Ax6m1NosBKni4NnJpj/7Z7rBCl32EED6sxm+JqQXF4fxA9uKKJklFND7qMYeY61f/j3jZcVITNAkUidzmE14wB2oePn9gk3e/LBF6rTdcvLsPfADl3tuFW6tV652kA+dB9K7Di64qQ90bHguegWpN/FuOxOzwh5EkLc8Sr7L8YymJf+if5KtiMYIoVZ4Ol8dDGBOyohJQ08i9KeLJDNam68k8ctXibSfHHE+ROtcqOUaCzuuLZXm3yF467t/kbtpMVzYI1uyvmUKH2iF7MHmLsAHTbbsz+1rh4vOtXgvaaxAe642jqoc3EqmC3BwKq9ij8Hp4/ZWIzHySgqsxTX4/5DmZl7qv2LnO49t62Ow2FZzTw2tEOWyCqlpiLsP5+Iyqq8Cp8wbS1eIzun2BacMcdjE760luR5kMB96lxYVTEinedumoS8rwJPW0m+9UjalUXYlksjiKgqta8mpfbllibO9MLbPPW9IaPleEvG7P/jVsY7cFkkSWXUnK08
				
				%URL%
				https://www.gnu.org/software/grep/
				
				%LICENSE%
				GPL3
				
				%ARCH%
				x86_64
				
				%BUILDDATE%
				1629412344
				
				%PACKAGER%
				Sébastien Luttringer <seblu@seblu.net>
				
				%DEPENDS%
				glibc
				pcre
				
				%MAKEDEPENDS%
				texinfo"#
			).unwrap(),
			Package {
				_id: None,
				_rev: None,
				channel: PackageChannel::Pacman,
				name: String::from("grep"),
				version: String::from("3.7-1"),
				filename: Some(String::from("grep-3.7-1-x86_64.pkg.tar.zst")),
				description: Some(String::from("A string search utility")),
				size: Some(String::from("254020")),
				md5sum: Some(String::from("df80335ae92442feb3c9bfdb9d91eff6")),
				sha256sum: Some(String::from(
					"eca12ceaef774299ed7022de9b00dd7ce379186da4c223cf357fc157a9064da6"
				)),
				upstream_url: Some(String::from("https://www.gnu.org/software/grep/")),
				license: Some(String::from("GPL3")),
				arch: Some(String::from("x86_64")),
				build_date: Some(String::from("1629412344")),
				analysis: None,
				worker: None,
			}
		);
	}
}
