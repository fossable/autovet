use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub enum FindingSeverity {
	#[default]
	Info,
	Warning,
	Critical,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct Finding {
	pub severity: FindingSeverity,

	pub message: String,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct Analysis {
	pub _id: String,

	pub _rev: String,

	pub package_id: String,

	pub name: String,

	pub progress: u32,

	pub start_time: u64,

	pub end_time: u64,

	pub findings: Vec<Finding>,
}

pub trait StaticAnalyzer {
	fn analyze(path: String) -> Analysis;
}
