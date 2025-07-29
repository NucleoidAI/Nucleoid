"""Event management for Nucleoid AI runtime."""

from typing import List, Optional
from .types import Event


class EventManager:
    """Manages runtime events."""
    
    def __init__(self):
        self._events: List[Event] = []
    
    def add(self, topic: str, data: str) -> None:
        """Add an event to the event list."""
        event = Event(topic=topic, data=data)
        self._events.append(event)
    
    def list(self) -> List[Event]:
        """Get all events."""
        return self._events.copy()
    
    def clear(self) -> None:
        """Clear all events."""
        self._events = []
    
    def get_events_by_topic(self, topic: str) -> List[Event]:
        """Get events filtered by topic."""
        return [event for event in self._events if event.topic == topic]