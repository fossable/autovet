pub mod pacman;
pub mod query;
pub mod test;

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    Test {

        /// The package to test
        package: String,
    },

    SubmitEnvironment {},

    /// Query the autovet public API
    Query {

        /// The package channel to query
        #[clap(long)]
        channel: Option<String>,

        /// The package to query
        #[clap(long)]
        package: Option<String>,

        /// The package version to query
        #[clap(long)]
        version: Option<String>,
    },

    /// A pacman wrapper
    #[clap(trailing_var_arg = true)]
    Pacman {
        /// Regular pacman arguments
        #[clap(multiple_values = true, allow_hyphen_values = true)]
        arguments: Vec<String>,
    },
}
