pub mod pacman;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	env_logger::init_from_env(env_logger::Env::new());
	crate::pacman::sync()
}
