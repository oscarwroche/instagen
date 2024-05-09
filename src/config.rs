use clap::Parser;

/// Server for the instagen ap
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Location for the key file
    #[arg(short, long)]
    pub key_file: String,

    /// Location for the cert file
    #[arg(short, long)]
    pub cert_file: String,
}
