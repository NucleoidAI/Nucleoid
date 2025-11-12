"""
State module for managing runtime state.
This is a stub implementation that will be expanded later.
"""

from typing import Any


class State:
    """Manages runtime state for Nucleoid"""

    def __init__(self) -> None:
        self._state: dict[str, Any] = {}

    def clear(self) -> None:
        """Clear all runtime state"""
        self._state.clear()

    def get(self, key: str) -> Any:
        """Get a state value"""
        return self._state.get(key)

    def set(self, key: str, value: Any) -> None:
        """Set a state value"""
        self._state[key] = value


# Singleton instance
_state = State()


def clear() -> None:
    """Clear the state"""
    _state.clear()


def get(key: str) -> Any:
    """Get a state value"""
    return _state.get(key)


def set(key: str, value: Any) -> None:
    """Set a state value"""
    _state.set(key, value)
