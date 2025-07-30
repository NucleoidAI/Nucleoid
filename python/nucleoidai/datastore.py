"""Data store for Nucleoid AI runtime."""

import json
import pickle
from pathlib import Path
from typing import List, Optional, Any, Dict

from .types import Data, Config
from .cache import Cache


class DataStore:
    """Main data store for runtime execution data."""
    
    def __init__(self):
        self.cache = Cache()
        self._initialized = False
    
    def init(self, config: Optional[Config] = None) -> None:
        """Initialize the data store."""
        if config is None:
            from .config import get_config
            config = get_config()
        
        self.cache.init(config)
        self._initialized = True
    
    def clear(self) -> None:
        """Clear all stored data."""
        if not self._initialized:
            self.init()
        self.cache.clear()
    
    def read(self) -> List[Data]:
        """Read all stored data."""
        if not self._initialized:
            self.init()
        
        raw_data = self.cache.read()
        processed_data = []
        
        for data in raw_data:
            # Revive the data (convert serialized objects back)
            if isinstance(data, dict):
                processed_data.append(self._revive_data(data))
            else:
                processed_data.append(data)
        
        return processed_data
    
    def write(self, data: Data) -> Data:
        """Write data to the store."""
        if not self._initialized:
            self.init()
        
        return self.cache.write(data)
    
    def tail(self, n: int) -> List[Data]:
        """Get the last n entries."""
        if not self._initialized:
            self.init()
        
        return self.cache.tail(n)
    
    def _revive_data(self, data: Dict[str, Any]) -> Data:
        """Revive serialized data back to proper objects."""
        # Convert dictionary back to Data object
        try:
            return Data(
                string=data.get("string", ""),
                declarative=data.get("declarative"),
                result=data.get("result"),
                time=data.get("time"),
                date=data.get("date"),
                error=data.get("error"),
                events=data.get("events", [])
            )
        except Exception:
            # If conversion fails, return as-is
            return data