pub struct PackageDeclaration {
	pub channels: Vec<String>,
}

pub struct DynamicAnalysis {
	pub syscalls: Option<SyscallSelection>,

	pub tests: Vec<Test>,
}

pub struct StaticAnalysis {
	pub analyzers: Vec<String>,
	pub syscalls: Option<SyscallSelection>,
}

pub struct Test {
	pub syscalls: Option<SyscallSelection>,

	pub test: String,
}

pub struct SyscallSelection {
	/// Allowed system call regexes
	pub allow: Vec<String>,

	/// Disallowed system call regexes
	pub disallow: Vec<String>,
}
