#[derive(clap::Parser, Debug)]
pub struct Config {
    /// The connection URL for the Postgres database this application should use.
    #[clap(long, env)]
    pub font_path: String,
}