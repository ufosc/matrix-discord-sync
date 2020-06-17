use std::sync::{Arc, Mutex, mpsc};

// use tokio::sync::mpsc;

use log::{error, info};

mod discord;
mod matrix;

#[tokio::main]
async fn main() {
    kankyo::init().expect("Failed to load .env file.\nSee .env.example in project root.");
    env_logger::init();

    let (tx_to_matrix, rx_matrix) = mpsc::channel();

    // let locked_rx: Arc<Mutex<mpsc::Receiver<discord::DiscordToMatrixMsg>>> = Arc::new(Mutex::new(rx_matrix));

    info!("Starting Discord bot");
    let discord_thread = std::thread::spawn(move || {
        discord::init(tx_to_matrix);
    });

    info!("Starting Matrix bot");
    // let matrix_thread = std::thread::spawn(move || {
        // let lock = locked_rx.lock();
        matrix::init(&rx_matrix).await;
    // });

    discord_thread.join().expect("Error joining Discord bot thread.");
    // matrix_thread.join().expect("Error joining matrix");
}
