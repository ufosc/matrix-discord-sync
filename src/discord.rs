use std::{env, sync::Arc, sync::mpsc};

use serenity::framework::standard::{
    macros::{command, group},
    CommandResult,
};
use serenity::framework::StandardFramework;
use serenity::{
    model::{
        channel::Channel,
        channel::GuildChannel,
    },
    prelude::*,
};

use log::{error, info};

pub enum ChannelEvent {
    NewChannel(GuildChannel),
    UpdatedChannel(GuildChannel, GuildChannel),
    DeletedChannel(GuildChannel),
}

struct SharedDataContainer;

impl TypeMapKey for SharedDataContainer {
    type Value = Arc<Mutex<mpsc::Sender<ChannelEvent>>>;
}

struct Handler;
impl EventHandler for Handler {
    fn channel_create(&self, ctx: Context, channel: Arc<RwLock<GuildChannel>>) {
        let c = unwrap_and_copy_channel(&channel);
        let tx = get_tx_clone(&ctx);
        std::thread::spawn(move || {
            handle_new_channel(c, &tx);
        });
    }

    fn channel_update(&self, ctx: Context, old: Option<Channel>, new: Channel) {
        if old.is_none() {
            return;
        }

        let wrap_old_channel = match old.unwrap() {
            Channel::Guild(x) => x,
            _ => return,
        };

        if let Channel::Guild(wrap_new_channel) = new {
            let old_channel = unwrap_and_copy_channel(&wrap_old_channel);
            let new_channel = unwrap_and_copy_channel(&wrap_new_channel);
            let tx = get_tx_clone(&ctx);
            std::thread::spawn(move || {
                handle_updated_channel(old_channel, new_channel, &tx); 
            });
        } else {
            return;
        }
    }

    fn channel_delete(&self, ctx: Context, channel: Arc<RwLock<GuildChannel>>) {
        let c = unwrap_and_copy_channel(&channel);
        let tx = get_tx_clone(&ctx);
        std::thread::spawn(move || {
            handle_deleted_channel(c, &tx);
        });
    }
}

pub fn unwrap_and_copy_channel(wrapped_channel: &Arc<RwLock<GuildChannel>>) -> GuildChannel {
    (*wrapped_channel.read()).clone()
}

pub fn get_tx_clone(ctx: &Context) -> mpsc::Sender<ChannelEvent> {
    let data = ctx.data.read();
    let tx_mutex = data.get::<SharedDataContainer>().expect("Error retrieving SharedDataContainer");
    let tx = tx_mutex.lock();
    tx.clone()
}

pub fn handle_new_channel(channel: GuildChannel, tx: &mpsc::Sender<ChannelEvent>) {
    info!(
        "Channel {} created with ID {} and server {}\n",
        channel.name, channel.id, channel.guild_id
    );
    match tx.send(ChannelEvent::NewChannel(channel)) {
        Ok(_) => (),
        Err(_) => error!("Error sending NewChannel event to matrix"),
    };
}

pub fn handle_updated_channel(old_channel: GuildChannel, new_channel: GuildChannel, tx: &mpsc::Sender<ChannelEvent>) {
    info!(
        "Channel {} updated with ID {} and server {}. Now called {}\n",
        old_channel.name,
        old_channel.id,
        old_channel.guild_id,
        new_channel.name()
    );
    match tx.send(ChannelEvent::UpdatedChannel(old_channel, new_channel)) {
        Ok(_) => (),
        Err(_) => error!("Error sending UpdatedChannel event to matrix"),
    };
}

pub fn handle_deleted_channel(channel: GuildChannel, tx: &mpsc::Sender<ChannelEvent>) {
    info!(
        "Channel {} deleted with ID {} and server {}\n",
        channel.name, channel.id, channel.guild_id
    );
    match tx.send(ChannelEvent::DeletedChannel(channel)) {
        Ok(_) => (),
        Err(_) => error!("Error sending DeletedChannel event to matrix"),
    };
}

pub fn init(tx: mpsc::Sender<ChannelEvent>) {
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");
    let mut client = Client::new(&token, Handler).expect("Error creating Discord client");

    {
        let mut data = client.data.write();
        data.insert::<SharedDataContainer>(Arc::from(Mutex::new(tx.clone())));
    }

    client.with_framework(
        StandardFramework::new().configure(|c| c.prefix("~")),
    );
    if let Err(why) = client.start() {
        println!("Err with client: {:?}", why);
    }
}
