"""Runtime processor for Nucleoid AI statements."""

import time
from datetime import datetime
from typing import Any, List, Optional

from .types import Data, Options, Result, Event
from .config import get_config, get_options
from .event import EventManager
from .datastore import DataStore
from .lib.statement import Statement
from .transaction import TransactionManager
from .stack import StackProcessor
from .context import ContextManager


class RuntimeProcessor:
    """Main runtime processor for executing statements."""
    
    def __init__(self, context_manager: ContextManager):
        self.context_manager = context_manager
        self.event_manager = EventManager()
        self.datastore = DataStore()
        self.transaction_manager = TransactionManager()
        self.stack_processor = StackProcessor()
        
        self.default_options = Options(
            declarative=False,
            details=False
        )
    
    def process(self, statement_string: str, options: Optional[Options] = None) -> Any:
        """
        Process a statement string and return the result.
        
        Args:
            statement_string: The statement to process
            options: Processing options
            
        Returns:
            The result of processing the statement
        """
        if options is None:
            options = Options()
        
        # Merge options with defaults and config
        config_options = get_options()
        merged_options = Options(
            declarative=options.declarative if options.declarative is not None 
                       else config_options.declarative if config_options.declarative is not None
                       else self.default_options.declarative,
            details=options.details if options.details is not None
                   else config_options.details if config_options.details is not None  
                   else self.default_options.details
        )
        
        before = time.time()
        result = None
        error = False
        
        try:
            # Compile statement to AST nodes
            statements = Statement.compile(statement_string)
            
            if not statements:
                return None
            
            # Process within transaction
            self.transaction_manager.start()
            result = self.stack_processor.process(statements, None, merged_options)
            self.transaction_manager.end()
            
        except Exception as err:
            self.transaction_manager.rollback()
            error = True
            result = err
        
        # Collect execution metadata
        events = self.event_manager.list()
        end_time = time.time()
        execution_time = (end_time - before) * 1000  # Convert to milliseconds
        
        # Create data object
        data = Data(
            string=statement_string,
            declarative=merged_options.declarative,
            result=result,
            time=execution_time,
            date=datetime.now(),
            error=error,
            events=events or []
        )
        
        # Store execution data
        self.datastore.write(data)
        self.event_manager.clear()
        
        # Return appropriate result
        if merged_options.details:
            return data
        else:
            if error:
                raise result
            
            # Extract value from result if it has one
            if hasattr(result, 'value'):
                return result.value
            return result