"""State management for Nucleoid AI declarative runtime."""

from typing import Any, Dict, List, Optional, Union, Callable
from .transaction import get_transaction_manager
from .lib.serialize import serialize_json, deserialize_json


class Scope:
    """Represents a scope in the declarative runtime."""
    
    def __init__(self):
        self.prior: Optional['Scope'] = None
        self.block: Dict[str, Any] = {}
        self.root: Optional['Scope'] = None
        self.local: Dict[str, Any] = {}
        self.instance: Optional[Any] = None
        self.graph: Dict[str, Any] = {}
        self.callback: List[Callable[[], None]] = []
        self.instances: Dict[str, Any] = {}
        self.object: Optional[Dict[str, str]] = None


class State:
    """Global state for the declarative runtime."""
    
    def __init__(self):
        self.data: Dict[str, Any] = {
            "classes": []
        }
    
    def __getitem__(self, key: str) -> Any:
        return self.data.get(key)
    
    def __setitem__(self, key: str, value: Any) -> None:
        self.data[key] = value
    
    def __delitem__(self, key: str) -> None:
        if key in self.data:
            del self.data[key]
    
    def __contains__(self, key: str) -> bool:
        return key in self.data
    
    def get(self, key: str, default: Any = None) -> Any:
        return self.data.get(key, default)
    
    def clear(self) -> None:
        """Clear all state except classes."""
        self.data.clear()
        self.data["classes"] = []


# Global state instance
global_state = State()


class StateManager:
    """Manages state operations for the declarative runtime."""
    
    def __init__(self):
        self.state = global_state
    
    def assign(self, scope: Scope, variable: str, evaluation: Union[str, Any], json: bool = True) -> Any:
        """Assign a value to a state variable."""
        transaction_manager = get_transaction_manager()
        
        # Extract value from evaluation
        if hasattr(evaluation, 'value'):
            eval_value = evaluation.value
        else:
            eval_value = str(evaluation)
        
        # Process the value
        if json:
            try:
                # In a real implementation, this would safely evaluate the expression
                # For now, we'll store the evaluation string
                value = eval_value
                if isinstance(value, str):
                    try:
                        # Try to parse as JSON/Python literal
                        value = eval(value) if value.strip() else None
                    except:
                        # Keep as string if evaluation fails
                        pass
                
                # Serialize the value
                serialized_value = serialize_json(value)
                processed_value = deserialize_json(serialized_value)
            except Exception:
                processed_value = eval_value
        else:
            processed_value = eval_value
        
        # Register with transaction system
        state_key = f"state.{variable}"
        if transaction_manager.is_active():
            transaction_manager.record_change("assign", {
                "key": state_key,
                "value": processed_value
            })
        
        # Store in state
        self.state[variable] = processed_value
        return processed_value
    
    def call(self, scope: Scope, fn: str, args: Optional[List[Any]] = None) -> Any:
        """Call a function on the state."""
        if args is None:
            args = []
        
        # Parse function call
        if '.' in fn:
            parts = fn.split('.')
            obj_name = parts[0]
            method_name = parts[1]
            
            if obj_name in self.state.data:
                obj = self.state.data[obj_name]
                
                if method_name == "push" and isinstance(obj, list):
                    for arg in args:
                        obj.append(arg)
                    return obj
                elif method_name == "pop" and isinstance(obj, list):
                    return obj.pop() if obj else None
                elif hasattr(obj, method_name):
                    method = getattr(obj, method_name)
                    if callable(method):
                        return method(*args)
        
        return None
    
    def expression(self, scope: Scope, evaluation: Dict[str, str]) -> Any:
        """Evaluate an expression."""
        try:
            expr_value = evaluation.get('value', '')
            # In a real implementation, this would safely evaluate the expression
            # For now, return the expression string
            return expr_value
        except Exception:
            return None
    
    def delete(self, scope: Scope, variable: str) -> bool:
        """Delete a state variable."""
        try:
            if variable in self.state.data:
                del self.state.data[variable]
                return True
            return False
        except Exception:
            return False
    
    def throw_exception(self, scope: Scope, exception: str) -> None:
        """Throw an exception."""
        raise Exception(exception)
    
    def clear(self) -> None:
        """Clear the global state."""
        self.state.clear()


# Module-level functions for backward compatibility
def assign(scope: Scope, variable: str, evaluation: Union[str, Any], json: bool = True) -> Any:
    """Assign a value to a state variable."""
    manager = StateManager()
    return manager.assign(scope, variable, evaluation, json)

def call(scope: Scope, fn: str, args: Optional[List[Any]] = None) -> Any:
    """Call a function on the state."""
    manager = StateManager()
    return manager.call(scope, fn, args)

def expression(scope: Scope, evaluation: Dict[str, str]) -> Any:
    """Evaluate an expression."""
    manager = StateManager()
    return manager.expression(scope, evaluation)

def delete(scope: Scope, variable: str) -> bool:
    """Delete a state variable."""
    manager = StateManager()
    return manager.delete(scope, variable)

def throw_exception(scope: Scope, exception: str) -> None:
    """Throw an exception."""
    manager = StateManager()
    manager.throw_exception(scope, exception)

def clear() -> None:
    """Clear the global state."""
    global_state.clear()