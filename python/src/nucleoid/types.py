"""
Type definitions for Nucleoid runtime.
"""
from typing import Any, Dict, List, Optional
from datetime import datetime
from dataclasses import dataclass, field


@dataclass
class Options:
    """Runtime execution options."""

    declarative: bool = False
    details: bool = False


@dataclass
class PortConfig:
    """Port configuration."""

    terminal: int = 8448
    cluster: int = 4000
    openapi: int = 3000


@dataclass
class DataConfig:
    """Data configuration."""

    encryption: bool = True


@dataclass
class Config:
    """Runtime configuration."""

    path: Optional[str] = None
    port: PortConfig = field(default_factory=PortConfig)
    options: Dict[str, Any] = field(default_factory=dict)
    cache: bool = False
    data: DataConfig = field(default_factory=DataConfig)
    id: Optional[str] = None
    test: bool = False


@dataclass
class Event:
    """Event data structure."""

    topic: str
    data: str


@dataclass
class Result:
    """Execution result structure."""

    nuc: List[Any] = field(default_factory=list)
    value: Any = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            '$nuc': self.nuc,
            'value': self.value
        }


@dataclass
class Data:
    """Execution data structure."""

    string: str
    result: Result
    time: float
    date: datetime
    declarative: bool = False
    error: bool = False
    events: List[Event] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'string': self.string,
            'declarative': self.declarative,
            'result': self.result.to_dict() if isinstance(self.result, Result) else self.result,
            'time': self.time,
            'date': self.date.isoformat() if isinstance(self.date, datetime) else self.date,
            'error': self.error,
            'events': [
                {'topic': e.topic, 'data': e.data} if isinstance(e, Event) else e
                for e in self.events
            ]
        }
