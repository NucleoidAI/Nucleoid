"""
Main Nucleoid runtime interface.

This module provides the primary API for the Nucleoid runtime, including start(), run(),
and register() functions. It corresponds to the TypeScript nucleoid.ts module.
"""

from typing import Any

from rich.console import Console
from rich.text import Text

from . import context
from .config import init as init_config
from .runtime import process
from .types import Config, Options

console = Console()


def start(config: Config | None = None) -> None:
    """
    Start the Nucleoid runtime with the given configuration.
    
    Args:
        config: Runtime configuration options
    """
    # Initialize configuration
    init_config(config)

    # Display startup message with rich formatting
    nucleoid_text = Text("Nucleoid", style="bright_green")
    nature_text = Text("🌎 Inspired by Nature", style="bright_blue")

    console.print("🌿 " + str(nucleoid_text) + " runtime is started")
    console.print(nature_text)
    console.print()


def run(statement: str, options: Options | None = None) -> Any:
    """
    Execute a Nucleoid statement and return the result.
    
    Args:
        statement: The Nucleoid statement string to execute
        options: Execution options
        
    Returns:
        The result of statement execution
    """
    return process(statement, options)


def register(fn: Any) -> None:
    """
    Register a function in the Nucleoid context for declarative execution.
    
    Args:
        fn: Function to register (should have a string representation)
    """
    # Load the function definition into context with declarative options
    context.load([
        {
            "definition": str(fn),
            "options": {"declarative": True},
        }
    ])


# Create default export object
class NucleoidDefault:
    """Default export object for Nucleoid runtime."""

    start = staticmethod(start)
    run = staticmethod(run)
    register = staticmethod(register)


# Default export instance
default = NucleoidDefault()
