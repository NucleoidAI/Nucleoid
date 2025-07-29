"""Main Nucleoid AI runtime module."""

import sys
from typing import Any, Dict, Optional, Callable

from colorama import Fore, Style, init as colorama_init

from .config import init as config_init, Config
from .runtime import RuntimeProcessor
from .context import ContextManager
from .types import Options, RuntimeValue

# Initialize colorama for cross-platform colored output
colorama_init()

# Global instances
_runtime_processor: Optional[RuntimeProcessor] = None
_context_manager: Optional[ContextManager] = None


def start(config: Optional[Dict[str, Any]] = None) -> None:
    """Start the Nucleoid runtime environment."""
    global _runtime_processor, _context_manager
    
    # Initialize configuration
    if config is None:
        config = {}
    
    config_init(config)
    
    # Initialize runtime components
    _context_manager = ContextManager()
    _runtime_processor = RuntimeProcessor(_context_manager)
    
    # Print startup message
    print(f"🌿 {Fore.GREEN}Nucleoid{Style.RESET_ALL} runtime is started")
    print(f"{Fore.BLUE}🌎 Inspired by Nature{Style.RESET_ALL}\n")


def run(statement: str, options: Optional[Dict[str, Any]] = None) -> Any:
    """
    Run a declarative statement in the Nucleoid runtime.
    
    Args:
        statement: The statement to execute
        options: Optional execution options
        
    Returns:
        The result of the statement execution
    """
    global _runtime_processor
    
    if _runtime_processor is None:
        start()  # Auto-start if not already started
    
    if options is None:
        options = {}
    
    # Convert dict to Options if needed
    if isinstance(options, dict):
        options = Options(**options)
    
    return _runtime_processor.process(statement, options)


def register(fn: Callable) -> None:
    """
    Register a declarative function with the runtime.
    
    Args:
        fn: The function to register as declarative
    """
    global _context_manager
    
    if _context_manager is None:
        start()  # Auto-start if not already started
    
    # Load the function as a declarative definition
    _context_manager.load([{
        "definition": _get_function_source(fn),
        "options": {"declarative": True}
    }])


def _get_function_source(fn: Callable) -> str:
    """Get the source code of a function."""
    import inspect
    try:
        return inspect.getsource(fn)
    except OSError:
        # Fallback for functions defined interactively
        return f"def {fn.__name__}():\n    pass"


# Export the main functions
__all__ = ["start", "run", "register"]