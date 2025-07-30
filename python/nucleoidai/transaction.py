"""Transaction management for Nucleoid AI runtime."""

from typing import Optional, Any, Dict, List
from contextlib import contextmanager


class TransactionManager:
    """Manages transactions for runtime operations."""
    
    def __init__(self):
        self._active_transaction: Optional[Dict[str, Any]] = None
        self._transaction_stack: List[Dict[str, Any]] = []
    
    def start(self) -> None:
        """Start a new transaction."""
        transaction = {
            "id": len(self._transaction_stack),
            "changes": [],
            "rollback_data": {}
        }
        
        if self._active_transaction:
            self._transaction_stack.append(self._active_transaction)
        
        self._active_transaction = transaction
    
    def end(self) -> None:
        """End the current transaction (commit)."""
        if not self._active_transaction:
            return
        
        # Commit changes (in this simple implementation, we just clear)
        self._active_transaction = None
        
        if self._transaction_stack:
            self._active_transaction = self._transaction_stack.pop()
    
    def rollback(self) -> None:
        """Rollback the current transaction."""
        if not self._active_transaction:
            return
        
        # In a full implementation, we would revert changes here
        # For now, we just clear the transaction
        self._active_transaction = None
        
        if self._transaction_stack:
            self._active_transaction = self._transaction_stack.pop()
    
    def is_active(self) -> bool:
        """Check if there's an active transaction."""
        return self._active_transaction is not None
    
    def record_change(self, operation: str, data: Any) -> None:
        """Record a change in the current transaction."""
        if self._active_transaction:
            self._active_transaction["changes"].append({
                "operation": operation,
                "data": data
            })
    
    @contextmanager
    def transaction(self):
        """Context manager for automatic transaction handling."""
        self.start()
        try:
            yield
            self.end()
        except Exception:
            self.rollback()
            raise


# Global transaction manager instance
_transaction_manager = TransactionManager()

def get_transaction_manager() -> TransactionManager:
    """Get the global transaction manager instance."""
    return _transaction_manager