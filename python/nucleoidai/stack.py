"""Stack processing for Nucleoid AI runtime."""

from typing import Any, List, Optional

from .types import Options, Result


class StackProcessor:
    """Processes statements in a stack-based manner."""
    
    def __init__(self):
        self._stack: List[Any] = []
    
    def process(self, statements: List[Any], context: Optional[Any], options: Options) -> Result:
        """
        Process a list of statements.
        
        Args:
            statements: List of AST statements to process
            context: Optional execution context
            options: Processing options
            
        Returns:
            Result object with processed value
        """
        if not statements:
            return Result(nuc=[], value=None)
        
        result_value = None
        nuc_data = []
        
        # Process each statement
        for stmt in statements:
            # In a full implementation, this would evaluate the AST
            # For now, we'll do basic processing
            try:
                if hasattr(stmt, 'value'):
                    result_value = stmt.value
                elif hasattr(stmt, 'id') and hasattr(stmt.id, 'name'):
                    # Simple variable reference
                    result_value = stmt.id.name
                else:
                    result_value = str(stmt)
                
                nuc_data.append(stmt)
                
            except Exception as e:
                # Handle processing errors
                result_value = f"Error processing statement: {e}"
        
        return Result(nuc=nuc_data, value=result_value)
    
    def push(self, value: Any) -> None:
        """Push a value onto the stack."""
        self._stack.append(value)
    
    def pop(self) -> Any:
        """Pop a value from the stack."""
        if self._stack:
            return self._stack.pop()
        return None
    
    def peek(self) -> Any:
        """Peek at the top value on the stack."""
        if self._stack:
            return self._stack[-1]
        return None
    
    def clear(self) -> None:
        """Clear the stack."""
        self._stack = []
    
    def size(self) -> int:
        """Get the size of the stack."""
        return len(self._stack)