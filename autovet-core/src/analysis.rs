

pub enum FindingSeverity {
	Info,
	Warning,
	Critical,
}

pub struct Finding {
	pub severity: FindingSeverity,

	pub message: String,
}

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
