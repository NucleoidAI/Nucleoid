"""
State module for managing runtime state.
"""
from typing import Any, Dict, List, Optional, Union


# Serializable value types
SerializableValue = Union[
    str,
    int,
    float,
    bool,
    None,
    Dict[str, Any],
    List[Any]
]


class State:
    """Runtime state manager."""

    def __init__(self) -> None:
        """Initialize state."""
        self.classes: List[Any] = []
        self._state: Dict[str, Any] = {}

    def __getitem__(self, key: str) -> Any:
        """Get state value."""
        if key == 'classes':
            return self.classes
        return self._state.get(key)

    def __setitem__(self, key: str, value: Any) -> None:
        """Set state value."""
        if key == 'classes':
            self.classes = value
        else:
            self._state[key] = value

    def __delitem__(self, key: str) -> None:
        """Delete state value."""
        if key in self._state:
            del self._state[key]

    def __contains__(self, key: str) -> bool:
        """Check if key exists."""
        return key == 'classes' or key in self._state

    def get(self, key: str, default: Any = None) -> Any:
        """Get state value with default."""
        if key == 'classes':
            return self.classes
        return self._state.get(key, default)

    def set(self, key: str, value: Any) -> None:
        """Set a state value."""
        if key == 'classes':
            self.classes = value
        else:
            self._state[key] = value

    def keys(self) -> List[str]:
        """Get all state keys."""
        return ['classes'] + list(self._state.keys())

    def assign(
        self,
        scope: Any,
        variable: str,
        evaluation: Union[str, Dict[str, str]],
        json: bool = True
    ) -> SerializableValue:
        """
        Assign a value to state.

        Args:
            scope: Execution scope
            variable: Variable name
            evaluation: Value or evaluation object
            json: Whether to serialize as JSON

        Returns:
            Assigned value
        """
        from . import transaction
        from .lib.serialize import serialize

        value: SerializableValue

        if json:
            eval_value = evaluation if isinstance(evaluation, str) else evaluation.get('value', '')
            # In Python, we use ast.literal_eval for safe evaluation
            import ast
            try:
                parsed_value = ast.literal_eval(eval_value)
                value = serialize(parsed_value, 'state')
            except (ValueError, SyntaxError):
                value = eval_value
        else:
            value = evaluation if isinstance(evaluation, str) else evaluation.get('value', '')

        # Register transaction
        return transaction.register_variable(variable, value, self._state)

    def call(
        self,
        scope: Any,
        fn: str,
        args: Optional[List[SerializableValue]] = None
    ) -> SerializableValue:
        """
        Call a state function.

        Args:
            scope: Execution scope
            fn: Function name
            args: Function arguments

        Returns:
            Function result
        """
        if args is None:
            args = []

        # Get the function from state
        if hasattr(self, fn):
            func = getattr(self, fn)
            return func(*args)
        elif fn in self._state and callable(self._state[fn]):
            return self._state[fn](*args)

        return None

    def expression(
        self,
        scope: Any,
        evaluation: Dict[str, str]
    ) -> SerializableValue:
        """
        Evaluate an expression.

        Args:
            scope: Execution scope
            evaluation: Evaluation object with value

        Returns:
            Evaluated result
        """
        import ast
        eval_value = evaluation.get('value', '')

        try:
            return ast.literal_eval(eval_value)
        except (ValueError, SyntaxError):
            return eval_value

    def delete(self, scope: Any, variable: str) -> bool:
        """
        Delete a state variable.

        Args:
            scope: Execution scope
            variable: Variable name

        Returns:
            True if deleted, False otherwise
        """
        if variable in self._state:
            del self._state[variable]
            return True
        return False

    def throw(self, scope: Any, exception: str) -> None:
        """
        Throw an exception.

        Args:
            scope: Execution scope
            exception: Exception to throw
        """
        raise Exception(exception)

    def clear(self) -> None:
        """Clear all state."""
        self._state.clear()
        self.classes = []


# Create singleton instance - using $ as the state object
state_dollar = State()
state_instance = state_dollar


def assign(
    scope: Any,
    variable: str,
    evaluation: Union[str, Dict[str, str]],
    json: bool = True
) -> SerializableValue:
    """
    Assign a value to state.

    Args:
        scope: Execution scope
        variable: Variable name
        evaluation: Value or evaluation object
        json: Whether to serialize as JSON

    Returns:
        Assigned value
    """
    return state_instance.assign(scope, variable, evaluation, json)


def call(
    scope: Any,
    fn: str,
    args: Optional[List[SerializableValue]] = None
) -> SerializableValue:
    """
    Call a state function.

    Args:
        scope: Execution scope
        fn: Function name
        args: Function arguments

    Returns:
        Function result
    """
    return state_instance.call(scope, fn, args)


def expression(scope: Any, evaluation: Dict[str, str]) -> SerializableValue:
    """
    Evaluate an expression.

    Args:
        scope: Execution scope
        evaluation: Evaluation object with value

    Returns:
        Evaluated result
    """
    return state_instance.expression(scope, evaluation)


def delete(scope: Any, variable: str) -> bool:
    """
    Delete a state variable.

    Args:
        scope: Execution scope
        variable: Variable name

    Returns:
        True if deleted, False otherwise
    """
    return state_instance.delete(scope, variable)


def throw(scope: Any, exception: str) -> None:
    """
    Throw an exception.

    Args:
        scope: Execution scope
        exception: Exception to throw
    """
    state_instance.throw(scope, exception)


def clear() -> None:
    """Clear all state."""
    state_instance.clear()
