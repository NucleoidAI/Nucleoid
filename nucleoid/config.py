"""
Configuration management for the Nucleoid runtime.

This module handles configuration initialization, directory setup, and runtime configuration
management. It provides similar functionality to the TypeScript config.ts module.
"""

import json
from pathlib import Path
from typing import Any
from uuid import uuid4

from dotenv import load_dotenv
from pydantic import BaseModel

from .lib.deep import deep_merge
from .types import Config


class RuntimeConfig(BaseModel):
    """Internal runtime configuration with full type safety."""
    path: str
    port: dict[str, int]
    options: dict[str, Any]
    cache: bool
    data: dict[str, bool]
    id: str | None = None


# Global configuration defaults
HOME = str(Path.home())

DEFAULT_CONFIG: RuntimeConfig = RuntimeConfig(
    path=f"{HOME}/.nuc",
    port={
        "terminal": 8448,
        "cluster": 4000,
        "openapi": 3000,
    },
    options={},
    cache=False,
    data={
        "encryption": True,
    },
)

# Global config instance
_config: RuntimeConfig = DEFAULT_CONFIG.model_copy()


def init(config: Config | None = None, test: bool = False,
         cli_args: dict[str, Any] | None = None) -> RuntimeConfig:
    """
    Initialize the Nucleoid configuration.
    
    Args:
        config: Configuration overrides
        test: Whether running in test mode
        cli_args: Command line arguments
        
    Returns:
        The initialized configuration
    """
    global _config

    # Merge configuration
    config = config or {}
    merged_config = deep_merge(DEFAULT_CONFIG.model_dump(), config)
    _config = RuntimeConfig(**merged_config)

    # Create necessary directories
    config_path = Path(_config.path)
    config_path.mkdir(parents=True, exist_ok=True)

    # Load environment variables
    env_file = config_path / ".env"
    if env_file.exists():
        load_dotenv(env_file)

    # Create subdirectories
    for subdir in ["data", "openapi", "native", "extensions"]:
        (config_path / subdir).mkdir(parents=True, exist_ok=True)

    # Create nucleoid.py entry point (equivalent to nucleoid.js)
    nucleoid_py = config_path / "nucleoid.py"
    nucleoid_py.write_text(
        '"""Nucleoid runtime module loader."""\n'
        '_nucleoid = None\n\n'
        'def get_nucleoid(nucleoid=None):\n'
        '    """Get or set the nucleoid runtime instance."""\n'
        '    global _nucleoid\n'
        '    if nucleoid is not None:\n'
        '        _nucleoid = nucleoid\n'
        '    return _nucleoid\n'
    )

    # Handle test mode and ID generation
    if test:
        _config.id = config.get("id") or str(uuid4())
        _config.cache = True
    else:
        # Try to load existing config
        config_file = config_path / "config.json"
        if config_file.exists():
            try:
                with open(config_file) as f:
                    saved_config = json.load(f)
                    merged_config = deep_merge(_config.model_dump(), saved_config)
                    _config = RuntimeConfig(**merged_config)
            except (json.JSONDecodeError, Exception):
                pass  # Continue with current config if loading fails

    # Handle ID assignment
    cli_args = cli_args or {}
    runtime_id = cli_args.get("id") or _config.id

    if not runtime_id:
        default_file = config_path / "default"
        try:
            runtime_id = default_file.read_text().strip()
        except (FileNotFoundError, Exception):
            runtime_id = str(uuid4())

    if runtime_id:
        default_file = config_path / "default"
        default_file.write_text(str(runtime_id))

    _config.id = runtime_id

    # Apply CLI overrides
    if cli_args.get("terminal_port"):
        _config.port["terminal"] = cli_args["terminal_port"]

    if cli_args.get("cluster_port"):
        _config.port["cluster"] = cli_args["cluster_port"]

    return _config


def get_config() -> RuntimeConfig:
    """Get the current runtime configuration."""
    return _config


def get_options() -> dict[str, Any]:
    """Get the current runtime options."""
    return _config.options
