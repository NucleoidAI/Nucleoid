"""
Core runtime processing for Nucleoid statements.

This module handles the execution of statements, manages the runtime environment,
and provides the main processing loop. It corresponds to the TypeScript runtime.ts module.
"""

import time
from datetime import datetime
from typing import Any

from .config import get_config
from .datastore import datastore
from .event import event_manager
from .lib.statement import Statement
from .stack import process_stack
from .transaction import transaction_manager
from .types import Data, Options, Result

# Default options for statement processing
DEFAULT_OPTIONS: Options = {
    "declarative": False,
    "details": False,
}


def process(statement: str, options: Options | None = None) -> Data | Any:
    """
    Process a Nucleoid statement string and return the result.
    
    This is the main entry point for executing Nucleoid statements. It handles
    parsing, execution, transaction management, and result formatting.
    
    Args:
        statement: The Nucleoid statement string to execute
        options: Execution options (declarative mode, details, etc.)
        
    Returns:
        Either a Data object with full details or just the computed value
        
    Raises:
        Various exceptions depending on statement execution errors
    """
    # Merge options with defaults and config
    final_options = {**DEFAULT_OPTIONS}
    config_options = get_config().options
    if config_options:
        final_options.update(config_options)
    if options:
        final_options.update(options)

    start_time = time.time()
    result: Any = None
    error = False

    try:
        # Compile the statement string into AST nodes
        statements = Statement.compile(statement)

        if not statements:
            return None

        # Start transaction tracking
        transaction_manager.start()

        # Process the statements through the stack processor
        result = process_stack(statements, None, final_options)

        # Commit the transaction
        transaction_manager.end()

    except Exception as err:
        # Rollback transaction on error
        transaction_manager.rollback()
        error = True
        result = err

    # Collect events and timing information
    events = event_manager.list()
    end_time = time.time()
    execution_time = int((end_time - start_time) * 1000)  # Convert to milliseconds

    # Create the data object
    data = Data(
        string=statement,
        declarative=final_options.get("declarative"),
        result=Result(nuc=result.get("$nuc", []) if isinstance(result, dict) else [],
                     value=result.get("value") if isinstance(result, dict) else result),
        time=execution_time,
        date=datetime.now(),
        error=error,
        events=events,
    )

    # Store the execution data
    datastore.write(data)

    # Clear events for next execution
    event_manager.clear()

    # Return based on options
    if final_options.get("details"):
        return data
    else:
        if error:
            raise result

        # Return the computed value
        if isinstance(result, dict) and "value" in result:
            return result["value"]
        return result


# Export the process function as the default
default = type('RuntimeDefault', (), {'process': process})()
