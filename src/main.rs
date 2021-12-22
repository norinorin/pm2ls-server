extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod decoder;
mod player;

use player::Player;
use std::error::Error;
use tokio::net::UdpSocket;

const LOG_LEVEL_VAR: &str = "LOG_LEVEL";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var(LOG_LEVEL_VAR).is_err() {
        std::env::set_var(LOG_LEVEL_VAR, "INFO");
    }

    pretty_env_logger::init_custom_env(LOG_LEVEL_VAR);

    let socket = UdpSocket::bind("0.0.0.0:7619").await?;

    info!("Listening on: {}", socket.local_addr()?);

    let player = Player::from_socket(socket);

    player.run().await?;

    Ok(())
}
