"""
Transaction management for the Nucleoid runtime.

This module provides transaction support for rollback capabilities during
statement execution, similar to the TypeScript transaction.ts module.
"""

from dataclasses import dataclass
from typing import Any, Union


@dataclass
class VariableTransaction:
    """Transaction for global variable changes."""
    variable: str
    before: Any


@dataclass
class ObjectTransaction:
    """Transaction for object property changes."""
    object: dict[str, Any]
    property: str
    before: Any


Transaction = Union[VariableTransaction, ObjectTransaction]


class TransactionManager:
    """Manages transactions for rollback support."""

    def __init__(self) -> None:
        self._transactions: list[Transaction] = []

    def start(self) -> None:
        """Start a new transaction session."""
        self._transactions.clear()

    def end(self) -> list[Transaction]:
        """End the transaction session and return the transaction list."""
        result = self._transactions.copy()
        self._transactions.clear()
        return result

    def register_variable(self, variable: str, value: Any, global_scope: dict[str, Any]) -> Any:
        """
        Register a variable change in the transaction.
        
        Args:
            variable: Variable name
            value: New value
            global_scope: Global scope dictionary to modify
            
        Returns:
            The new value that was set
        """
        # Capture old value
        before = global_scope.get(variable)

        # Apply new value
        global_scope[variable] = value

        # Record transaction
        transaction = VariableTransaction(variable=variable, before=before)
        self._transactions.append(transaction)

        return value

    def register_object(self, obj: dict[str, Any], property: str, value: Any) -> None:
        """
        Register an object property change in the transaction.
        
        Args:
            obj: Object to modify
            property: Property name
            value: New value
        """
        # Capture old value
        before = obj.get(property)

        # Record transaction
        transaction = ObjectTransaction(object=obj, property=property, before=before)
        self._transactions.append(transaction)

        # Apply new value
        obj[property] = value

    def rollback(self, global_scope: dict[str, Any]) -> None:
        """
        Rollback all transactions in reverse order.
        
        Args:
            global_scope: Global scope dictionary for variable rollbacks
        """
        while self._transactions:
            transaction = self._transactions.pop()

            if isinstance(transaction, VariableTransaction):
                # Rollback global variable
                if transaction.before is None:
                    global_scope.pop(transaction.variable, None)
                else:
                    global_scope[transaction.variable] = transaction.before

            elif isinstance(transaction, ObjectTransaction):
                # Rollback object property
                if transaction.before is None:
                    transaction.object.pop(transaction.property, None)
                else:
                    transaction.object[transaction.property] = transaction.before


# Global transaction manager instance
transaction_manager = TransactionManager()


def register(obj: dict[str, Any], key: str, value: Any) -> None:
    """
    Register a transaction for object property changes.
    
    Args:
        obj: Object to modify
        key: Property key
        value: New value
    """
    # Store previous value if it exists
    previous = obj.get(key)
    
    # Apply new value
    obj[key] = value
    
    # Create transaction record for rollback
    transaction = ObjectTransaction(object=obj, property=key, before=previous)
    transaction_manager._transactions.append(transaction)
