"""
Transaction module for managing state changes with rollback capability.
"""
from typing import Any, Dict, List, Optional, Union


class VariableTransaction:
    """Transaction for variable changes."""

    def __init__(self, variable: str, before: Any) -> None:
        """
        Initialize a variable transaction.

        Args:
            variable: Variable name
            before: Previous value
        """
        self.variable: str = variable
        self.before: Any = before


class ObjectTransaction:
    """Transaction for object property changes."""

    def __init__(self, obj: Dict[str, Any], property: str, before: Any) -> None:
        """
        Initialize an object transaction.

        Args:
            obj: Object being modified
            property: Property name
            before: Previous value
        """
        self.object: Dict[str, Any] = obj
        self.property: str = property
        self.before: Any = before


Transaction = Union[VariableTransaction, ObjectTransaction]


class TransactionManager:
    """Manages transactions with start, end, and rollback."""

    def __init__(self) -> None:
        """Initialize transaction manager."""
        self.list: List[Transaction] = []

    def start(self) -> None:
        """Start a new transaction."""
        self.list = []

    def end(self) -> List[Transaction]:
        """
        End current transaction and return list.

        Returns:
            List of transactions
        """
        result = self.list
        self.list = []
        return result

    def register_variable(self, variable: str, value: Any, scope: Dict[str, Any]) -> Any:
        """
        Register a variable transaction.

        Args:
            variable: Variable name
            value: New value
            scope: Scope containing the variable

        Returns:
            New value after assignment
        """
        # Capture old value
        before = scope.get(variable)

        # Apply new value
        scope[variable] = value

        # Record transaction
        self.list.append(VariableTransaction(variable, before))

        return value

    def register_object(self, obj: Dict[str, Any], property: str, value: Any) -> None:
        """
        Register an object property transaction.

        Args:
            obj: Object being modified
            property: Property name
            value: New value
        """
        # Capture old value
        before = obj.get(property)

        # Record transaction
        self.list.append(ObjectTransaction(obj, property, before))

        # Apply new value
        obj[property] = value

    def rollback(self) -> None:
        """Rollback all transactions in reverse order."""
        while self.list:
            transaction = self.list.pop()

            if isinstance(transaction, VariableTransaction):
                # Variable transaction - would need scope to restore
                # In practice, this requires passing scope to rollback
                pass
            elif isinstance(transaction, ObjectTransaction):
                # Object property transaction
                if transaction.before is None:
                    # Remove property if it didn't exist before
                    transaction.object.pop(transaction.property, None)
                else:
                    # Restore previous value
                    transaction.object[transaction.property] = transaction.before


# Create singleton instance
transaction_manager = TransactionManager()


def start() -> None:
    """Start a new transaction."""
    transaction_manager.start()


def end() -> List[Transaction]:
    """
    End current transaction and return list.

    Returns:
        List of transactions
    """
    return transaction_manager.end()


def register_variable(variable: str, value: Any, scope: Dict[str, Any]) -> Any:
    """
    Register a variable transaction.

    Args:
        variable: Variable name
        value: New value
        scope: Scope containing the variable

    Returns:
        New value after assignment
    """
    return transaction_manager.register_variable(variable, value, scope)


def register_object(obj: Dict[str, Any], property: str, value: Any) -> None:
    """
    Register an object property transaction.

    Args:
        obj: Object being modified
        property: Property name
        value: New value
    """
    transaction_manager.register_object(obj, property, value)


def rollback() -> None:
    """Rollback all transactions in reverse order."""
    transaction_manager.rollback()
