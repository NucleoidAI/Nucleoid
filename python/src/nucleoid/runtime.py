"""
Runtime module for processing code statements.
"""
from typing import Any, Dict, Optional
import time
from datetime import datetime


class Options:
    """Runtime execution options."""

    def __init__(
        self,
        declarative: bool = False,
        details: bool = False,
        **kwargs: Any
    ) -> None:
        """
        Initialize options.

        Args:
            declarative: Whether to run in declarative mode
            details: Whether to return detailed execution data
            **kwargs: Additional options
        """
        self.declarative: bool = declarative
        self.details: bool = details
        self.__dict__.update(kwargs)


class Data:
    """Execution data result."""

    def __init__(
        self,
        string: str,
        declarative: bool,
        result: Any,
        time: float,
        date: datetime,
        error: bool,
        events: list
    ) -> None:
        """
        Initialize execution data.

        Args:
            string: Input code string
            declarative: Whether run in declarative mode
            result: Execution result
            time: Execution time in milliseconds
            date: Execution timestamp
            error: Whether an error occurred
            events: List of events
        """
        self.string: str = string
        self.declarative: bool = declarative
        self.result: Any = result
        self.time: float = time
        self.date: datetime = date
        self.error: bool = error
        self.events: list = events

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        # Handle exception serialization
        result_value = self.result
        if isinstance(self.result, Exception):
            result_value = {
                'error': type(self.result).__name__,
                'message': str(self.result)
            }

        return {
            'string': self.string,
            'declarative': self.declarative,
            'result': result_value,
            'time': self.time,
            'date': self.date.isoformat() if isinstance(self.date, datetime) else self.date,
            'error': self.error,
            'events': self.events,
        }


DEFAULT_OPTIONS = Options()


class Runtime:
    """Runtime processor for code execution."""

    def process(self, string: str, options: Optional[Dict[str, Any]] = None) -> Any:
        """
        Process code string.

        Args:
            string: Code to execute
            options: Execution options

        Returns:
            Execution result or detailed data
        """
        # Lazy imports to avoid circular dependencies
        from . import config
        from . import event
        from . import datastore
        from .lib.statement import statement_instance
        from . import transaction
        from . import stack

        # Merge options
        opts_dict = {
            'declarative': DEFAULT_OPTIONS.declarative,
            'details': DEFAULT_OPTIONS.details,
        }

        # Add config options
        config_opts = config.config_instance.options
        if config_opts:
            opts_dict.update(config_opts)

        # Add provided options
        if options:
            opts_dict.update(options)

        opts = Options(**opts_dict)

        before = time.time()
        result: Any = None
        error = False

        try:
            statements = statement_instance.compile(string)

            if not statements:
                return None

            transaction.start()
            result = stack.stack_instance.process(statements, None, opts)
            transaction.end()

        except Exception as err:
            transaction.rollback()
            error = True
            result = err

        events = event.event_instance.list()
        date = datetime.now()
        elapsed = (time.time() - before) * 1000  # Convert to milliseconds

        data = Data(
            string=string,
            declarative=opts.declarative,
            result=result,
            time=elapsed,
            date=date,
            error=error,
            events=events,
        )

        datastore.datastore_instance.write(data.to_dict())
        event.event_instance.clear()

        if opts.details:
            return data.to_dict()
        else:
            if error:
                raise result

            # Return result value if it has a value attribute
            if hasattr(result, 'value'):
                return result.value
            return result


# Create singleton instance
runtime_instance = Runtime()
