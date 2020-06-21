from asyncio.queues import Queue
from typing import Optional

from discord.abc import GuildChannel
from discord.channel import TextChannel
from discord.ext import commands

from .util import EventType, ToMatrixMsg, Bridge


class DiscordWatcher(commands.Cog):
    def __init__(self, bot, to_discord_queue: Queue, to_matrix_queue: Queue):
        self.bot = bot
        self.to_discord_queue = to_discord_queue
        self.to_matrix_queue = to_matrix_queue

    def topic_or_default(self, topic: Optional[str]) -> str:
        return topic if topic else "No Topic"

    @commands.command()
    @commands.has_role('officer')
    async def sync(self, ctx: commands.Context):
        await self.to_matrix_queue.put(
            ToMatrixMsg(
                event_type=EventType.MANUAL_SYNC,
                bridge=Bridge(
                    guild_id=ctx.channel.guild.id,
                    channel_id=ctx.channel.id,
                    channel_name=ctx.channel.name,
                    channel_topic=self.topic_or_default(ctx.channel.topic)
                )
            )
        )

    @commands.Cog.listener()
    async def on_guild_channel_create(self, channel: GuildChannel):
        if isinstance(channel, TextChannel):
            await self.to_matrix_queue.put(
                ToMatrixMsg(
                    event_type=EventType.NEW_CHANNEL,
                    bridge=Bridge(
                        channel_name=channel.name,
                        channel_id=channel.id,
                        guild_id=channel.guild.id,
                        channel_topic=self.topic_or_default(channel.topic)
                    )
                ))
