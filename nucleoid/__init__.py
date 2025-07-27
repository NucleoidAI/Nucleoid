"""
Nucleoid - Declarative, logic-based, contextual runtime for Neuro-Symbolic AI.

A symbolic AI implementation that manages both data and logic under the same runtime 
environment, creating a knowledge graph that tracks relationships between declarative statements.
"""

from .nucleoid import register, run, start
from .types import Config, Data, Event, Options

__version__ = "0.7.10"
__author__ = "Can Mingir"
__license__ = "Apache-2.0"

__all__ = [
    "start",
    "run",
    "register",
    "Config",
    "Options",
    "Data",
    "Event",
]

# Default export equivalent
default = type('DefaultExport', (), {
    'start': start,
    'run': run,
    'register': register
})()
