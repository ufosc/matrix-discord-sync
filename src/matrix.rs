use std::{env, sync::mpsc};
use std::convert::TryFrom;

use ruma_client::{Client, Session, api::r0, HttpsClient};
use ruma_client::identifiers::user_id::UserId;
use serenity::model::channel::GuildChannel;
use serenity::model::id::{ChannelId, GuildId};

use log::{error, info};

use crate::discord::{DiscordToMatrixMsg, ChannelEvent};
use std::borrow::Cow;


pub async fn handle_discord_to_matrix_msg(msg: DiscordToMatrixMsg, client: &HttpsClient) {
    info!("Received new DiscordToMatrixMsg");
    let _response = match msg.event {
        ChannelEvent::NewChannel(channel) => handle_new_channel(channel, client).await,
        // ChannelEvent::UpdatedChannel(old_channel, new_channel) => handle_updated_channel(old_channel, new_channel),
        // ChannelEvent::DeletedChannel(channel) => handle_deleted_channel(channel, msg.http),
        _ => Err(String::from("Unhandled Discord Event.")),
    };
}

pub async fn handle_new_channel(channel: GuildChannel, client: &HttpsClient) -> Result<String, String> {
    let bridge_name = generate_bridge_name(channel.guild_id, channel.id);
    let resp = create_room(bridge_name, client).await;
    match resp {
        Ok(matrix_name) => Ok(format!("Created bridged channel {}", matrix_name)),
        Err(_) => Err(String::from("Error creating bridged room.")),
    }
}

pub fn generate_bridge_name(guild_id: GuildId, channel_id: ChannelId) -> String {
    return format!("_discord_{}_{}", guild_id, channel_id);
}

pub async fn create_room(room_name: String, client: &HttpsClient) -> Result<String, ()> {
    info!("Creating new room with name {}", room_name);
    let resp = client.request(r0::room::create_room::Request { 
        creation_content: None,
        initial_state: Vec::new(),
        invite: vec![UserId::try_from(Cow::from("@hjarrell:ufopensource.club")).unwrap()],
        invite_3pid: Vec::new(),
        is_direct: None,
        name: None,
        power_level_content_override: None,
        preset: None,
        room_alias_name: Some(room_name),
        room_version: None,
        topic: None,
        visibility: Some(r0::room::Visibility::Public)
    }).await;

    match resp {
        Ok(r) => {
            info!("Created room {} successfully!", r.room_id.to_string());
            Ok(r.room_id.to_string())
        },
        Err(e) => {
            error!("Error received creating room {}: ", e);
            Err(())
        },
    }
}

pub async fn init(rx: &mpsc::Receiver<DiscordToMatrixMsg>) {
    let homeserver_url = env::var("MATRIX_HOME_SERVER").expect("Expected a MATRIX_HOME_SERVER in the environment").parse().unwrap();
    let access_token = env::var("MATRIX_ACCESS_TOKEN").expect("Expected a MATRIX_ACCESS_TOKEN in the environment");
    let _work = async {
        let session = Session {access_token, identification: None};
        info!("Matrix bot started!");
        let client = Client::https(homeserver_url, Some(session));

        loop {
            info!("In main loop.");
            let msg = rx.recv();
            if msg.is_err() {
                error!("Error receiving message from discord->matrix mpsc.");
                break;
            } else {
                handle_discord_to_matrix_msg(msg.unwrap(), &client).await;
            }
        }
    }.await;
}
