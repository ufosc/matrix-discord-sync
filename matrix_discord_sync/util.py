from enum import Enum


class EventType(Enum):
    NEW_CHANNEL = 1
    UPDATE_CHANNEL = 2
    DELETED_CHANNEL = 3
    MANUAL_SYNC = 4


class Bridge:
    def __init__(self, guild_id: int, channel_id: int, channel_name: str, channel_topic: str):
        self.channel_name = channel_name
        self.channel_id = channel_id
        self.guild_id = guild_id
        self.channel_topic = channel_topic

    def make_room_link(self, homeserver: str) -> str:
        """
        Returns a link to the bridged channel with the name expected by the AppServices
        """
        return f"#_discord_{self.guild_id}_{self.channel_id}:{homeserver}"


class ToMatrixMsg:
    def __init__(self, event_type: EventType, bridge: Bridge):
        self.event_type = event_type
        self.bridge = bridge


class Subscriber:
    def __init__(self, user_id: str):
        self.user_id = user_id
