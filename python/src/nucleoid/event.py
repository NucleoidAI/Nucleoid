"""
Event module for tracking runtime events.
"""
import json
from typing import Any, Dict, List, Optional, Union


EventData = Dict[str, Union[str, int, bool, None]]


class Event:
    """Event tracker for runtime events."""

    def __init__(self) -> None:
        """Initialize event tracker."""
        self.events: List[Dict[str, str]] = []

    def event(self, name: str, data: EventData) -> None:
        """
        Record an event.

        Args:
            name: Event name
            data: Event data
        """
        self.events.append({
            'name': name,
            'data': json.dumps(data)
        })

    def list(self) -> Optional[List[Dict[str, str]]]:
        """
        Get list of events.

        Returns:
            List of events or None if empty
        """
        if self.events:
            return self.events
        return None

    def clear(self) -> None:
        """Clear all events."""
        self.events = []


# Create singleton instance
event_instance = Event()


def event(name: str, data: EventData) -> None:
    """
    Record an event to the singleton instance.

    Args:
        name: Event name
        data: Event data
    """
    event_instance.event(name, data)


def list() -> Optional[List[Dict[str, str]]]:
    """
    Get list of events from the singleton instance.

    Returns:
        List of events or None if empty
    """
    return event_instance.list()


def clear() -> None:
    """Clear all events from the singleton instance."""
    event_instance.clear()
