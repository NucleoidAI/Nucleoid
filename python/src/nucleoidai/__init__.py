"""
Nucleoid AI - Neuro-Symbolic AI with declarative runtime environment for Python.

This package provides a declarative runtime environment that combines symbolic logic
with neural networks for AI applications.
"""

from .nucleoid import start, run, register
from .express import create_app
from .config import Config
from .lib import openapi
from .lib import test
from . import datastore

__version__ = "0.7.10"
__author__ = "Can Mingir"
__email__ = "can@nucleoid.ai"
__license__ = "Apache-2.0"

__all__ = [
    "start",
    "run", 
    "register",
    "create_app",
    "Config",
    "openapi",
    "test",
    "datastore",
]