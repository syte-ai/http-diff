use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long, default_value = "./config.json")]
    pub config: String,

    #[arg(short, long, default_value = "./output")]
    pub output_directory: String,

    #[arg(short, long, default_value = "false")]
    pub enable_log: bool,
}
