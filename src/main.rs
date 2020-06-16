use std::sync::mpsc;

use log::{error, info};

mod discord;

fn main() {
    kankyo::init().expect("Failed to load .env file.\nSee .env.example in project root.");
    env_logger::init();

    let (tx_to_matrix, rx_matrix) = mpsc::channel();

    info!("Starting Discord bot");
    let discord_thread = std::thread::spawn(move || {
        discord::init(tx_to_matrix);
    });

    let matrix_thread = std::thread::spawn(move || {
        loop {
            match rx_matrix.recv() {
                Ok(event) => print_event(event),
                Err(_e) => {
                    error!("Error recieiving message from Discord.");
                    panic!("");
                }
            };
        }
    });

    discord_thread.join().expect("Error joining Discord bot thread.");
    matrix_thread.join().expect("error joining matrix");
}

fn print_event(event: discord::ChannelEvent) {
    match event {
        discord::ChannelEvent::NewChannel(nc) => info!("New channel event!"),
        discord::ChannelEvent::UpdatedChannel(nc1, nc2) => info!("Updated channel event!"),
        discord::ChannelEvent::DeletedChannel(nc) => info!("Deleted channel event!"),
    }
}
