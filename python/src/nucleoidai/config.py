"""Configuration management for Nucleoid AI."""

import json
import os
import uuid
from pathlib import Path
from typing import Any, Dict, Optional

from dotenv import load_dotenv

from .types import Config, PortConfig, DataConfig, Options
from .lib.deep import deep_merge


class ConfigManager:
    """Configuration manager for Nucleoid AI."""
    
    def __init__(self):
        self._config: Optional[Config] = None
        self._default_config = Config(
            path=str(Path.home() / ".nuc"),
            port=PortConfig(
                terminal=8448,
                cluster=4000,
                openapi=3000
            ),
            options=Options(),
            cache=False,
            data=DataConfig(encryption=True),
        )
    
    def init(self, config: Optional[Dict[str, Any]] = None) -> Config:
        """Initialize configuration with default and user settings."""
        if config is None:
            config = {}
        
        # Deep merge default config with user config
        merged_config = deep_merge(
            self._default_config.__dict__, 
            config
        )
        
        # Convert to Config object
        self._config = Config(
            path=merged_config.get("path", self._default_config.path),
            port=PortConfig(**merged_config.get("port", {})),
            options=Options(**merged_config.get("options", {})),
            cache=merged_config.get("cache", self._default_config.cache),
            data=DataConfig(**merged_config.get("data", {})),
            id=merged_config.get("id"),
            test=merged_config.get("test", False)
        )
        
        # Create directories
        config_path = Path(self._config.path)
        config_path.mkdir(parents=True, exist_ok=True)
        
        # Load environment variables
        env_file = config_path / ".env"
        if env_file.exists():
            load_dotenv(env_file)
        
        # Create subdirectories
        for subdir in ["data", "openapi", "native", "extensions"]:
            (config_path / subdir).mkdir(parents=True, exist_ok=True)
        
        # Create nucleoid.py module
        nucleoid_module = config_path / "nucleoid.py"
        nucleoid_module.write_text(
            '"""Auto-generated Nucleoid module."""\n'
            '_nucleoid = None\n\n'
            'def get_nucleoid(nucleoid=None):\n'
            '    """Get or set nucleoid instance."""\n'
            '    global _nucleoid\n'
            '    if nucleoid is not None:\n'
            '        _nucleoid = nucleoid\n'
            '    return _nucleoid\n'
        )
        
        # Handle test mode
        if self._config.test:
            self._config.id = config.get("id", str(uuid.uuid4()))
            self._config.cache = True
        else:
            # Try to load existing config
            config_file = config_path / "config.json"
            if config_file.exists():
                try:
                    with open(config_file, 'r') as f:
                        file_config = json.load(f)
                    # Merge with file config
                    merged = deep_merge(self._config.__dict__, file_config)
                    self._config = Config(**merged)
                except (json.JSONDecodeError, TypeError):
                    pass  # Use default config
        
        # Handle ID generation/loading
        if not self._config.id:
            default_file = config_path / "default"
            try:
                self._config.id = default_file.read_text().strip()
            except FileNotFoundError:
                self._config.id = str(uuid.uuid4())
        
        # Save default ID
        default_file = config_path / "default"
        default_file.write_text(self._config.id)
        
        return self._config
    
    def get_config(self) -> Config:
        """Get current configuration."""
        if self._config is None:
            return self.init()
        return self._config
    
    def get_options(self) -> Options:
        """Get current options."""
        return self.get_config().options or Options()


# Global configuration manager
_config_manager = ConfigManager()

# Export functions
def init(config: Optional[Dict[str, Any]] = None) -> Config:
    """Initialize configuration."""
    return _config_manager.init(config)

def get_config() -> Config:
    """Get current configuration."""
    return _config_manager.get_config()

def get_options() -> Options:
    """Get current options."""
    return _config_manager.get_options()