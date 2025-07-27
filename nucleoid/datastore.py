"""
Data storage interface for the Nucleoid runtime.

This module provides data persistence functionality, corresponding to
the TypeScript datastore integration.
"""

import json
from pathlib import Path
from typing import Any

from .config import get_config
from .types import Data


class Datastore:
    """Simple file-based datastore for Nucleoid execution data."""

    def __init__(self):
        self._data_path: Path | None = None

    @property
    def data_path(self) -> Path:
        """Get the data storage path."""
        if self._data_path is None:
            config = get_config()
            self._data_path = Path(config.path) / "data"
            self._data_path.mkdir(parents=True, exist_ok=True)
        return self._data_path

    def write(self, data: Data) -> None:
        """
        Write execution data to storage.
        
        Args:
            data: The execution data to store
        """
        try:
            # Create a simple JSON file for each execution
            # In a real implementation, this might use a proper database
            filename = f"execution_{int(data.date.timestamp() * 1000)}.json"
            file_path = self.data_path / filename

            # Convert Pydantic model to dict for JSON serialization
            data_dict = data.model_dump(mode='json')

            with open(file_path, 'w', encoding='utf-8') as f:
                json.dump(data_dict, f, indent=2, ensure_ascii=False)

        except Exception as e:
            # In production, this might log the error instead of silently failing
            print(f"Warning: Failed to write data to storage: {e}")

    def read_all(self) -> list[dict[str, Any]]:
        """
        Read all stored execution data.
        
        Returns:
            List of execution data dictionaries
        """
        data_list = []

        try:
            if not self.data_path.exists():
                return data_list

            for file_path in self.data_path.glob("execution_*.json"):
                try:
                    with open(file_path, encoding='utf-8') as f:
                        data = json.load(f)
                        data_list.append(data)
                except Exception as e:
                    print(f"Warning: Failed to read {file_path}: {e}")

        except Exception as e:
            print(f"Warning: Failed to read data directory: {e}")

        return data_list

    def clear(self) -> None:
        """Clear all stored data."""
        try:
            if self.data_path.exists():
                for file_path in self.data_path.glob("execution_*.json"):
                    file_path.unlink()
        except Exception as e:
            print(f"Warning: Failed to clear data: {e}")

    def tail(self, limit: int = 10) -> list[dict[str, Any]]:
        """
        Get the most recent execution data entries.
        
        Args:
            limit: Maximum number of entries to return
            
        Returns:
            List of recent execution data entries
        """
        all_data = self.read_all()
        
        # Sort by date (most recent first) and limit
        sorted_data = sorted(
            all_data, 
            key=lambda x: x.get('date', ''), 
            reverse=True
        )
        
        return sorted_data[:limit]


# Global datastore instance
datastore = Datastore()
