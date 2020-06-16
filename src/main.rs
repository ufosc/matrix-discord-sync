use log::{error, info};

mod discord;

fn main() {
    kankyo::init().expect("Failed to load .env file.\nSee .env.example in project root.");
    env_logger::init();

    info!("Starting Discord bot");
    let discord_thread = std::thread::spawn(move || {
        discord::init();
    });

    discord_thread.join().expect("Error joining Discord bot thread.");
}
