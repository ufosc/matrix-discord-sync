use std::{borrow::Cow, env, sync::Arc};

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

use log::info;

enum ChannelEvent {
    NewChannel(GuildChannel),
    UpdatedChannel(GuildChannel, GuildChannel),
    DeletedChannel(GuildChannel),
}

struct Handler;
impl EventHandler for Handler {
    fn channel_create(&self, _ctx: Context, channel: Arc<RwLock<GuildChannel>>) {
        let c = unwrap_and_copy_channel(&channel);
        handle_new_channel(c);
    }

    fn channel_update(&self, _ctx: Context, old: Option<Channel>, new: Channel) {
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
            handle_updated_channel(&old_channel, &new_channel);
        } else {
            return;
        }
    }

    fn channel_delete(&self, _ctx: Context, channel: Arc<RwLock<GuildChannel>>) {
        let c = unwrap_and_copy_channel(&channel);
        handle_deleted_channel(c);
    }
}

pub fn unwrap_and_copy_channel(wrapped_channel: &Arc<RwLock<GuildChannel>>) -> GuildChannel {
    (*wrapped_channel.read()).clone()
}

pub fn handle_new_channel(channel: GuildChannel) {
    info!(
        "Channel {} created with ID {} and server {}\n",
        channel.name, channel.id, channel.guild_id
    );
}

pub fn handle_updated_channel(new_channel: &GuildChannel, old_channel: &GuildChannel) {
    info!(
        "Channel {} updated with ID {} and server {}. Now called {}\n",
        new_channel.name,
        new_channel.id,
        new_channel.guild_id,
        old_channel.name()
    );
}

pub fn handle_deleted_channel(channel: GuildChannel) {
    info!(
        "Channel {} deleted with ID {} and server {}\n",
        channel.name, channel.id, channel.guild_id
    );
}

pub fn init() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");
    let mut client = Client::new(&token, Handler).expect("Error creating Discord client");
    client.with_framework(
        StandardFramework::new().configure(|c| c.prefix("~")),
    );
    if let Err(why) = client.start() {
        println!("Err with client: {:?}", why);
    }
}
