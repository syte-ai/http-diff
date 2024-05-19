use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long, default_value = "./configuration.json")]
    pub configuration: String,

    #[arg(short, long, default_value = "./output")]
    pub output_directory: String,

    #[arg(short, long, default_value = "false")]
    pub enable_log: bool,

    #[arg(long, default_value = "false")]
    pub headless: bool,
}
