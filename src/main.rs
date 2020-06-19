use std::sync::mpsc;

use log::{info};

mod discord;
mod matrix;

#[tokio::main]
async fn main() {
    kankyo::init().expect("Failed to load .env file.\nSee .env.example in project root.");
    env_logger::init();

    let (tx_to_matrix, rx_matrix) = mpsc::channel();

    info!("Starting Discord bot");
    let discord_thread = std::thread::spawn(move || {
        discord::init(tx_to_matrix);
    });

    info!("Starting Matrix bot");
    matrix::init(&rx_matrix).await;

    discord_thread.join().expect("Error joining Discord bot thread.");
}
