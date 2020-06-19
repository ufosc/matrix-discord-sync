use std::{env, sync::Arc, collections::HashSet, sync::mpsc};

use serenity::{
    http::Http,
    framework::{
        StandardFramework,
        standard::{
            Args,
            macros::{
                command,
                group,
            },
            CommandResult,
        },
    },
    model::{
        channel::Channel,
        channel::GuildChannel,
        channel::Message,
    },
    prelude::*,
};

use log::{error, info};

pub enum ChannelEvent {
    NewChannel(GuildChannel),
    UpdatedChannel(GuildChannel, GuildChannel),
    DeletedChannel(GuildChannel),
}

pub struct DiscordToMatrixMsg {
    pub event: ChannelEvent,
    pub http: Arc<Http>,
}

struct SharedDataContainer;

impl TypeMapKey for SharedDataContainer {
    type Value = Arc<Mutex<mpsc::Sender<DiscordToMatrixMsg>>>;
}

struct Handler;
impl EventHandler for Handler {
    fn channel_create(&self, ctx: Context, channel: Arc<RwLock<GuildChannel>>) {
        let c = unwrap_and_copy_channel(&channel);
        let tx = get_tx_clone(&ctx);
        let http = ctx.http.clone();
        std::thread::spawn(move || {
            let http = http.clone();
            handle_new_channel(http, c, &tx);
        });
    }

    fn channel_delete(&self, ctx: Context, channel: Arc<RwLock<GuildChannel>>) {
        let c = unwrap_and_copy_channel(&channel);
        let tx = get_tx_clone(&ctx);
        let http = ctx.http.clone();
        std::thread::spawn(move || {
            let http = http.clone();
            handle_deleted_channel(http, c, &tx);
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
            let http = ctx.http.clone();
            std::thread::spawn(move || {
                let http = http.clone();
                handle_updated_channel(http, old_channel, new_channel, &tx);
            });
        } else {
            return;
        }
    }
}

pub fn unwrap_and_copy_channel(wrapped_channel: &Arc<RwLock<GuildChannel>>) -> GuildChannel {
    (*wrapped_channel.read()).clone()
}

pub fn get_tx_clone(ctx: &Context) -> mpsc::Sender<DiscordToMatrixMsg> {
    let data = ctx.data.read();
    let tx_mutex = data.get::<SharedDataContainer>().expect("Error retrieving SharedDataContainer");
    let tx = tx_mutex.lock();
    tx.clone()
}

pub fn handle_new_channel(http: Arc<Http>, channel: GuildChannel, tx: &mpsc::Sender<DiscordToMatrixMsg>) {
    info!(
        "Channel {} created with ID {} and server {}\n",
        channel.name, channel.id, channel.guild_id
    );
    let msg = DiscordToMatrixMsg { event: ChannelEvent::NewChannel(channel), http };
    match tx.send(msg) {
        Ok(_) => (),
        Err(_) => error!("Error sending NewChannel event to matrix"),
    };
}

pub fn handle_updated_channel(http: Arc<Http>, old_channel: GuildChannel, new_channel: GuildChannel, tx: &mpsc::Sender<DiscordToMatrixMsg>) {
    info!(
        "Channel {} updated with ID {} and server {}. Now called {}\n",
        old_channel.name,
        old_channel.id,
        old_channel.guild_id,
        new_channel.name()
    );
    let msg = DiscordToMatrixMsg { event: ChannelEvent::UpdatedChannel(old_channel, new_channel), http };
    match tx.send(msg) {
        Ok(_) => (),
        Err(_) => error!("Error sending UpdatedChannel event to matrix"),
    };
}

pub fn handle_deleted_channel(http: Arc<Http>, channel: GuildChannel, tx: &mpsc::Sender<DiscordToMatrixMsg>) {
    info!(
        "Channel {} deleted with ID {} and server {}\n",
        channel.name, channel.id, channel.guild_id
    );
    let msg = DiscordToMatrixMsg { event: ChannelEvent::DeletedChannel(channel), http};
    match tx.send(msg) {
        Ok(_) => (),
        Err(_) => error!("Error sending DeletedChannel event to matrix"),
    };
}

#[command]
#[allowed_roles("officer")]
fn sync(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    if let Some(channel) = msg.channel(&ctx.cache) {
        match channel {
            Channel::Guild(gc) => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Requesting matrix bridged room...") {
                    error!("Error sending message: {:?}", why);
                } else {
                    let c = unwrap_and_copy_channel(&gc);
                    info!("Manual channel creation requested by: {} for {}", msg.author.name, c.name);
                    let tx = get_tx_clone(&ctx);
                    let http = ctx.http.clone();
                    std::thread::spawn(move || {
                        let http = http.clone();
                        handle_new_channel(http, c, &tx);
                    });
                }
            },
            _ => (),
        };
    }
    Ok(())
}

#[group]
#[commands(sync)]
struct Admin;

pub fn init(tx: mpsc::Sender<DiscordToMatrixMsg>) {
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");
    let mut client = Client::new(&token, Handler).expect("Error creating Discord client");

    {
        let mut data = client.data.write();
        data.insert::<SharedDataContainer>(Arc::from(Mutex::new(tx.clone())));
    }

    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        },
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c
                .owners(owners)
                .prefix("~")
            )
            .group(&ADMIN_GROUP),
    );

    info!("Discord bot started.");
    if let Err(why) = client.start() {
        println!("Err with client: {:?}", why);
    }
}
