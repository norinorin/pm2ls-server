extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod decoder;
mod player;

use clap::Parser;
use player::Player;
use std::error::Error;
use tokio::net::UdpSocket;

const LOG_LEVEL_VAR: &str = "LOG_LEVEL";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Whether or not retry to write if fails
    #[clap(short, long)]
    write_all: bool,

    /// Volume scaling in percent
    #[clap(short, long, default_value_t = 100)]
    volume: i16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if std::env::var(LOG_LEVEL_VAR).is_err() {
        std::env::set_var(LOG_LEVEL_VAR, "INFO");
    }

    pretty_env_logger::init_custom_env(LOG_LEVEL_VAR);

    let socket = UdpSocket::bind("0.0.0.0:7619").await?;

    info!("Listening on: {}", socket.local_addr()?);

    let player = Player::from_socket(socket, args.write_all, args.volume);

    player.run().await?;

    Ok(())
}
