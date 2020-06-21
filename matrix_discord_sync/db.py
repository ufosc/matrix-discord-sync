from typing import List

from sqlalchemy import (Column, String, Integer, Boolean, Text, DateTime, ForeignKey, Table, MetaData, select, and_)
from sqlalchemy.engine.base import Engine

from .util import Bridge, Subscriber


class MDSyncDatabase:
    db: Engine
    bridges: Table
    invite_subscribers: Table

    def __init__(self, db: Engine):
        self.db = db

        meta = MetaData()
        meta.bind = db
        self.bridges = Table("bridges", meta,
                             Column("guild_id", Integer, primary_key=True),
                             Column("channel_id", Integer, primary_key=True),
                             Column("channel_name", String(255), nullable=False),
                             Column("channel_topic", String(1024), nullable=False)
                             )
        self.invite_subscribers = Table("invite_subscribers", meta,
                                        Column("user_id", String(255), primary_key=True),
                                        )
        meta.create_all()

    def get_all_bridges(self) -> List[Bridge]:
        rows = self.db.execute(select([self.bridges]))
        return [Bridge(row[0], row[1], row[2], row[3]) for row in rows]

    def add_bridge(self, bridge: Bridge):
        with self.db.begin() as tx:
            result = tx.execute(self.bridges.insert().values(guild_id=bridge.guild_id, channel_id=bridge.channel_id,
                                                             channel_name=bridge.channel_name, channel_topic=bridge.channel_topic))

    def delete_bridge(self, bridge: Bridge):
        with self.db.begin() as tx:
            result = tx.execute(self.bridges.delete().where(and_(
                self.bridges.c.guild_id == bridge.guild_id,
                self.bridges.c.channel_id == bridge.channel_id
            )))

    def get_subscriptions(self) -> List[Subscriber]:
        rows = self.db.execute(select([self.invite_subscribers]))
        return [Subscriber(row[0]) for row in rows]

    def add_subscriber(self, subscriber: Subscriber):
        with self.db.begin() as tx:
            result = tx.execute(self.invite_subscribers.insert().values(user_id=subscriber.user_id))

    def delete_subscriber(self, subscriber: Subscriber):
        with self.db.begin() as tx:
            tx.execute(self.invite_subscribers.delete().where(self.invite_subscribers.c.user_id == subscriber.user_id))
