use std::default::Default;
use serde::{Serialize, Deserialize}

#[derive(Serialize, Deserialize, Default)]
pub struct PacmanPackage {

    pub name: String,

    pub filename: String,
    
    pub version: String,

    pub description: String,

    pub size: String,

    pub md5sum: String,

    pub sha256sum: String,

    pub url: String,

    pub license: String,

    pub arch: String,

    pub build_date: String,
}

impl PacmanPackage {
    pub fn from_desc(desc: String) -> PacmanPackage {
        let lines: Vec<&str> = desc.split("\n").collect();
        let mut package = PacmanPackage::default();

        for i in 0..lines.len() {
            match lines[i] {
                "%NAME%" => package.name = lines[i+1].to_string(),
                "%FILENAME%" => package.filename = lines[i+1].to_string(),
                "%VERSION%" => package.version = lines[i+1].to_string(),
                "%DESC%" => package.description = lines[i+1].to_string(),
                "%CSIZE%" => package.size = lines[i+1].to_string(),
                "%MD5SUM%" => package.md5sum = lines[i+1].to_string(),
                "%SHA256SUM%" => package.sha256sum = lines[i+1].to_string(),
                "%URL%" => package.url = lines[i+1].to_string(),
                "%LICENSE%" => package.license = lines[i+1].to_string(),
                "%ARCH%" => package.arch = lines[i+1].to_string(),
                "%BUILDDATE%" => package.build_date = lines[i+1].to_string(),
                _ => continue,
            }
        }

        package
    }
}