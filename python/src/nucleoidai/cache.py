"""Cache implementation for Nucleoid AI data store."""

import json
from typing import List, Any

from .types import Data, Config


class Cache:
    """In-memory cache for runtime data."""
    
    def __init__(self):
        self._cache: List[str] = []
    
    def init(self, config: Config) -> None:
        """Initialize the cache with configuration."""
        if config.cache:
            print("Cache is enabled")
        # In the Python version, we keep the cache in memory
        # In a production version, this could be backed by Redis, files, etc.
    
    def read(self) -> List[Data]:
        """Read all cached data."""
        data_list = []
        for data_str in self._cache:
            try:
                data_dict = json.loads(data_str)
                data_list.append(data_dict)
            except json.JSONDecodeError:
                continue
        return data_list
    
    def write(self, data: Data) -> Data:
        """Write data to cache."""
        # Convert Data object to dictionary for serialization
        data_dict = {
            "string": data.string,
            "declarative": data.declarative,
            "result": self._serialize_result(data.result),
            "time": data.time,
            "date": data.date.isoformat() if data.date else None,
            "error": data.error,
            "events": [{"topic": e.topic, "data": e.data} for e in (data.events or [])]
        }
        
        self._cache.append(json.dumps(data_dict, default=str))
        return data
    
    def clear(self) -> None:
        """Clear all cached data."""
        self._cache = []
    
    def tail(self, n: int = 10) -> List[Data]:
        """Get the last n entries."""
        data_list = self.read()
        return list(reversed(data_list))[:n]
    
    def _serialize_result(self, result: Any) -> Any:
        """Serialize result object for JSON storage."""
        if hasattr(result, '__dict__'):
            return result.__dict__
        elif hasattr(result, 'nuc') and hasattr(result, 'value'):
            return {
                "nuc": getattr(result, 'nuc', []),
                "value": getattr(result, 'value', None)
            }
        return result