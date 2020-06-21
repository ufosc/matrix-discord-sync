import asyncio
from asyncio.queues import Queue
from typing import List, Type, Tuple

from discord.ext import commands
from mautrix.util.config import BaseProxyConfig, ConfigUpdateHelper
from mautrix.types import RoomID, TextMessageEventContent, Format, MessageType
from maubot import Plugin
from maubot.handlers import command

from .db import MDSyncDatabase
from .discord_watcher import DiscordWatcher
from .util import Bridge, Subscriber, EventType, ToMatrixMsg


class MDSyncConfig(BaseProxyConfig):
    def do_update(self, helper: ConfigUpdateHelper) -> None:
        helper.copy("discord_token")
        helper.copy("link_room")


class MatrixDiscordSync(Plugin):
    async def start(self):
        await super().start()
        self.config.load_and_update()
        self.discord_bot = commands.Bot(
            command_prefix="~",
            description="Matrix Discord Sync (A Lame Sync Bot)",
            case_insensitive=True,
            command_not_found="Invalid command: {}")
        self.to_discord_queue = Queue()
        self.to_matrix_queue = Queue()
        self.homeserver = self.config["homeserver"]
        self.link_room = RoomID(self.config["link_room_id"])
        self.db = MDSyncDatabase(self.database)
        self.discord_bot.add_cog(DiscordWatcher(self.discord_bot, self.to_discord_queue, self.to_matrix_queue))
        self.discord_routine = asyncio.ensure_future(self.discord_bot.start(self.config["discord_token"]), loop=self.loop)
        self.event_routine = asyncio.ensure_future(self.matrix_event_loop(), loop=self.loop)

    async def stop(self):
        await super().stop()
        await self.discord_bot.close()

    async def matrix_event_loop(self):
        while True:
            msg: ToMatrixMsg = await self.to_matrix_queue.get()
            if msg.event_type == EventType.NEW_CHANNEL:
                await self.handle_new_channel(msg.bridge)
            elif msg.event_type == EventType.MANUAL_SYNC:
                await self.handle_new_channel(msg.bridge)

    async def handle_new_channel(self, bridge: Bridge):
        self.log.debug('Adding bridge')
        self.db.add_bridge(bridge)
        self.log.debug('Bridge added')

        await self.update_links()
    
    async def update_links(self):
        bridges = self.db.get_all_bridges()
        self.log.debug(f'Got {len(bridges)} bridges from DB')
        msg = self.format_plain_links(bridges)
        self.log.debug(f'Plain message {msg}')
        html_msg = self.format_html_links(bridges)
        self.log.debug(f'HTML message {html_msg}')

        await self.client.send_message(self.link_room, TextMessageEventContent(
            msgtype=MessageType.TEXT,
            body=msg,
            format=Format.HTML,
            formatted_body=html_msg
        ))
    
    def format_plain_links(self, bridges: List[Bridge]) -> str:
        msg = "List of bridged discord channels:\n"
        for b in bridges:
            msg += f"- \tChannel: #{b.channel_name}\n\tBridge: {b.make_room_link(self.homeserver)}\n"
        return msg

    def format_html_links(self, bridges: List[Bridge]) -> str:
        msg = "<h1>List of Bridged Discord Channels</h1>"
        for b in bridges:
            msg += f"<strong>#{b.channel_name}</strong> - {b.channel_topic} - {b.make_room_link(self.homeserver)}"
            msg += "<br/>"
        return msg

    @classmethod
    def get_config_class(cls) -> Type[BaseProxyConfig]:
        return MDSyncConfig
