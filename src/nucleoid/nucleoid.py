"""
Nucleoid - Main entry point for the Nucleoid runtime.
"""
from typing import Any, Callable, Dict, Optional, Union
from . import context
from . import config
from . import runtime
from .types import Config, Data, Options


def start(cfg: Optional[Dict[str, Any]] = None) -> None:
    """
    Start the Nucleoid runtime.

    Args:
        cfg: Configuration dictionary

    Example:
        >>> import nucleoid
        >>> nucleoid.start({'cache': True})
    """
    if cfg is None:
        cfg = {}

    config.init(cfg)

    try:
        print("🌿 Nucleoid runtime is started")
        print("🌎 Inspired by Nature\n")
    except UnicodeEncodeError:
        # Fallback if terminal doesn't support Unicode
        print("Nucleoid runtime is started")
        print("Inspired by Nature\n")

    # Note: process.init() is commented out in TypeScript version
    # from . import process
    # process.init()


def run(statement: str, options: Optional[Dict[str, Any]] = None) -> Union[Data, Any]:
    """
    Run a statement in the Nucleoid runtime.

    Args:
        statement: Code statement to execute
        options: Execution options (declarative, details)

    Returns:
        Execution result or detailed data

    Example:
        >>> nucleoid.run("let x = 5")
        >>> nucleoid.run("x + 10", {'details': True})
    """
    if options is None:
        options = {}

    return runtime.runtime_instance.process(statement, options)


def register(fn: Union[Callable, str]) -> None:
    """
    Register a function in declarative mode.

    Args:
        fn: Function to register (callable or string definition)

    Example:
        >>> def my_function():
        ...     return "hello"
        >>> nucleoid.register(my_function)
    """
    # Convert function to string if it's callable
    if callable(fn):
        import inspect
        definition = inspect.getsource(fn)
    else:
        definition = str(fn)

    context.load([
        {
            'definition': definition,
            'options': {'declarative': True}
        }
    ])


# Module-level exports
__all__ = ['start', 'run', 'register']
