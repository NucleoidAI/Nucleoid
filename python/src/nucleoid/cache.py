"""
Cache module for storing runtime data.
"""
import json
from typing import Any, Dict, List


class Cache:
    """In-memory cache for data storage."""

    def __init__(self) -> None:
        """Initialize cache."""
        self.cache: List[str] = []

    def init(self, config: Dict[str, Any]) -> None:
        """
        Initialize cache with configuration.

        Args:
            config: Configuration dictionary
        """
        if config.get('cache', False):
            print("Cache is enabled")

    def read(self) -> List[Dict[str, Any]]:
        """
        Read all cached data.

        Returns:
            List of data objects
        """
        return [json.loads(data) for data in self.cache]

    def write(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Write data to cache.

        Args:
            data: Data to cache

        Returns:
            Written data
        """
        self.cache.append(json.dumps(data))
        return data

    def clear(self) -> None:
        """Clear all cached data."""
        self.cache = []

    def tail(self, n: int = 10) -> List[Dict[str, Any]]:
        """
        Get last n entries from cache.

        Args:
            n: Number of entries to retrieve

        Returns:
            List of last n data objects
        """
        return [json.loads(data) for data in reversed(self.cache)][:n]


# Create singleton instance
cache_instance = Cache()


def init(config: Dict[str, Any]) -> None:
    """
    Initialize cache with configuration.

    Args:
        config: Configuration dictionary
    """
    cache_instance.init(config)


def read() -> List[Dict[str, Any]]:
    """
    Read all cached data.

    Returns:
        List of data objects
    """
    return cache_instance.read()


def write(data: Dict[str, Any]) -> Dict[str, Any]:
    """
    Write data to cache.

    Args:
        data: Data to cache

    Returns:
        Written data
    """
    return cache_instance.write(data)


def clear() -> None:
    """Clear all cached data."""
    cache_instance.clear()


def tail(n: int = 10) -> List[Dict[str, Any]]:
    """
    Get last n entries from cache.

    Args:
        n: Number of entries to retrieve

    Returns:
        List of last n data objects
    """
    return cache_instance.tail(n)
