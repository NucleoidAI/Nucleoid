"""
Event management for the Nucleoid runtime.

This module handles event tracking and management during statement execution.
"""

from .types import Event


class EventManager:
    """Manages runtime events during execution."""

    def __init__(self):
        self._events: list[Event] = []

    def add(self, topic: str, data: str) -> None:
        """Add an event to the current execution context."""
        event = Event(topic=topic, data=data)
        self._events.append(event)

    def list(self) -> list[Event]:
        """Get all events from the current execution context."""
        return self._events.copy()

    def clear(self) -> None:
        """Clear all events from the current execution context."""
        self._events.clear()


# Global event manager instance
event_manager = EventManager()
