"""Type definitions for Nucleoid AI."""

from typing import Any, Dict, List, Optional, Union
from datetime import datetime
from dataclasses import dataclass


@dataclass
class Options:
    """Options for runtime operations."""
    declarative: Optional[bool] = None
    details: Optional[bool] = None


@dataclass
class PortConfig:
    """Port configuration for services."""
    terminal: Optional[int] = None
    cluster: Optional[int] = None
    openapi: Optional[int] = None


@dataclass
class DataConfig:
    """Data configuration."""
    encryption: Optional[bool] = None


@dataclass
class Config:
    """Main configuration for Nucleoid runtime."""
    path: Optional[str] = None
    port: Optional[PortConfig] = None
    options: Optional[Options] = None
    cache: Optional[bool] = None
    data: Optional[DataConfig] = None
    id: Optional[str] = None
    test: Optional[bool] = None


@dataclass
class Event:
    """Event data structure."""
    topic: str
    data: str


@dataclass
class Result:
    """Runtime execution result."""
    nuc: List[Any]  # Equivalent to $nuc: $[]
    value: Any


@dataclass
class Data:
    """Data structure for runtime operations."""
    string: str
    declarative: Optional[bool] = None
    result: Optional[Result] = None
    time: Optional[float] = None
    date: Optional[datetime] = None
    error: Optional[bool] = None
    events: Optional[List[Event]] = None


# Type aliases for common use cases
ConfigDict = Dict[str, Any]
RuntimeValue = Union[str, int, float, bool, Dict[str, Any], List[Any], None]