pub mod pacman;
pub mod query;
pub mod test;

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    Test {
        executable: String,
    },

    UploadEnvironment {},

    Query {},

    /// A pacman wrapper
    #[clap(trailing_var_arg = true)]
    Pacman {
        /// Regular pacman arguments
        #[clap(multiple_values = true, allow_hyphen_values = true)]
        arguments: Vec<String>,
    },
}
