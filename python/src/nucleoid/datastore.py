"""
Datastore module for managing data persistence.
This is a stub implementation that will be expanded later.
"""

from typing import Any


class Datastore:
    """Manages data persistence for Nucleoid"""

    def __init__(self) -> None:
        self._data: dict[str, Any] = {}

    def clear(self) -> None:
        """Clear all data from the datastore"""
        self._data.clear()

    def get(self, key: str) -> Any:
        """Get a value from the datastore"""
        return self._data.get(key)

    def set(self, key: str, value: Any) -> None:
        """Set a value in the datastore"""
        self._data[key] = value


# Singleton instance
_datastore = Datastore()


def clear() -> None:
    """Clear the datastore"""
    _datastore.clear()


def get(key: str) -> Any:
    """Get a value from the datastore"""
    return _datastore.get(key)


def set(key: str, value: Any) -> None:
    """Set a value in the datastore"""
    _datastore.set(key, value)
