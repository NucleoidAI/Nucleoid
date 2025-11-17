"""
Datastore module for managing runtime data storage.
"""
from typing import Any, Dict, List, Optional
from . import cache


class Datastore:
    """Datastore wrapper around cache with data transformation."""

    def __init__(self) -> None:
        """Initialize datastore."""
        self.cache = cache.cache_instance

    def init(self, config: Optional[Dict[str, Any]] = None) -> None:
        """
        Initialize datastore with configuration.

        Args:
            config: Configuration dictionary with cache settings
        """
        if config is None:
            config = {'cache': True}
        self.cache.init(config)

    def clear(self) -> None:
        """Clear all stored data."""
        self.cache.clear()

    def read(self) -> List[Dict[str, Any]]:
        """
        Read all data with result transformation.

        Returns:
            List of data objects with revived $nuc nodes
        """
        from .lang.__nuc__.revive import revive

        result = []
        for data in self.cache.read():
            transformed = data.copy()
            if 'result' in data and isinstance(data['result'], dict):
                transformed['result'] = {
                    '$nuc': revive(data['result'].get('$nuc', [])),
                    'value': data['result'].get('value')
                }
            result.append(transformed)

        return result

    def write(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Write data to datastore.

        Args:
            data: Data to store

        Returns:
            Written data
        """
        return self.cache.write(data)

    def tail(self, n: int = 10) -> List[Dict[str, Any]]:
        """
        Get last n entries.

        Args:
            n: Number of entries to retrieve

        Returns:
            List of last n data objects
        """
        return self.cache.tail(n)


# Create singleton instance
datastore_instance = Datastore()


def init(config: Optional[Dict[str, Any]] = None) -> None:
    """
    Initialize datastore with configuration.

    Args:
        config: Configuration dictionary with cache settings
    """
    datastore_instance.init(config)


def clear() -> None:
    """Clear all stored data."""
    datastore_instance.clear()


def read() -> List[Dict[str, Any]]:
    """
    Read all data with result transformation.

    Returns:
        List of data objects with revived $nuc nodes
    """
    return datastore_instance.read()


def write(data: Dict[str, Any]) -> Dict[str, Any]:
    """
    Write data to datastore.

    Args:
        data: Data to store

    Returns:
        Written data
    """
    return datastore_instance.write(data)


def tail(n: int = 10) -> List[Dict[str, Any]]:
    """
    Get last n entries.

    Args:
        n: Number of entries to retrieve

    Returns:
        List of last n data objects
    """
    return datastore_instance.tail(n)
