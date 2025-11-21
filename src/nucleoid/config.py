"""
Configuration management for Nucleoid.
"""
import os
import json
import uuid as uuid_module
from typing import Dict, Any, Optional
from pathlib import Path


class Config:
    """Configuration class."""

    def __init__(self) -> None:
        """Initialize configuration with default values."""
        home = str(Path.home())

        self.path: str = os.path.join(home, '.nuc')
        self.port: Dict[str, int] = {
            'terminal': 8448,
            'cluster': 4000,
            'openapi': 3000,
        }
        self.options: Dict[str, Any] = {}
        self.cache: bool = False
        self.data: Dict[str, bool] = {
            'encryption': True,
        }
        self.id: Optional[str] = None


# Global config instance
_config = Config()


def init(config: Optional[Dict[str, Any]] = None) -> Config:
    """
    Initialize configuration.

    Args:
        config: Configuration overrides

    Returns:
        Initialized configuration
    """
    global _config

    if config is None:
        config = {}

    # Merge with defaults
    from .lib.deep import deep_merge

    default_dict = {
        'path': _config.path,
        'port': _config.port,
        'options': _config.options,
        'cache': _config.cache,
        'data': _config.data,
    }

    merged = deep_merge(default_dict, config)

    _config.path = merged.get('path', _config.path)
    _config.port = merged.get('port', _config.port)
    _config.options = merged.get('options', _config.options)
    _config.cache = merged.get('cache', _config.cache)
    _config.data = merged.get('data', _config.data)

    # Create directories
    os.makedirs(_config.path, exist_ok=True)

    # Load .env if exists
    env_path = os.path.join(_config.path, '.env')
    if os.path.exists(env_path):
        try:
            from dotenv import load_dotenv
            load_dotenv(env_path)
        except ImportError:
            pass  # dotenv not installed

    # Create subdirectories
    for subdir in ['data', 'openapi', 'native', 'extensions']:
        os.makedirs(os.path.join(_config.path, subdir), exist_ok=True)

    # Write nucleoid.js placeholder
    nucleoid_js_path = os.path.join(_config.path, 'nucleoid.js')
    with open(nucleoid_js_path, 'w') as f:
        f.write('/* eslint-disable */ let _nucleoid; module.exports = (nucleoid) => { if (nucleoid) { _nucleoid = nucleoid; } return _nucleoid; };')

    # Handle test mode
    if config.get('test'):
        _config.id = config.get('id') or str(uuid_module.uuid4())
        _config.cache = True
    else:
        # Try to load config.json
        config_json_path = os.path.join(_config.path, 'config.json')
        if os.path.exists(config_json_path):
            try:
                with open(config_json_path, 'r') as f:
                    json_config = json.load(f)
                    _config.port.update(json_config.get('port', {}))
                    _config.options.update(json_config.get('options', {}))
            except (json.JSONDecodeError, IOError):
                pass

    # Get or create ID
    id_value = config.get('id') or _config.id
    if not id_value:
        default_id_path = os.path.join(_config.path, 'default')
        try:
            with open(default_id_path, 'r') as f:
                id_value = f.read().strip()
        except (IOError, OSError):
            id_value = str(uuid_module.uuid4())

    # Write default ID
    if id_value:
        default_id_path = os.path.join(_config.path, 'default')
        with open(default_id_path, 'w') as f:
            f.write(id_value)

    _config.id = id_value

    # Handle CLI arguments (if available)
    # In Python, you'd use argparse for this
    # For now, we'll skip CLI arg handling

    return _config


def get_options() -> Dict[str, Any]:
    """
    Get configuration options.

    Returns:
        Configuration options dictionary
    """
    return _config.options


def get_config() -> Config:
    """
    Get current configuration.

    Returns:
        Current configuration instance
    """
    return _config


# Export config instance
config_instance = _config
