"""
Scope management for Nucleoid runtime execution.

This module provides the Scope class that handles variable and function scoping,
including nested scopes, instance management, and graph relationships.
"""

from typing import Any, Optional

from .lang.ast.identifier import Identifier
from .lang.estree.generator import append


class Scope:
    """
    Manages execution scope for variables, functions, and objects in Nucleoid runtime.
    
    Provides hierarchical scoping with support for:
    - Variable assignment and retrieval
    - Instance management
    - Graph relationships
    - Object context tracking
    """

    def __init__(self, prior: Optional['Scope'], block: Any) -> None:
        """
        Initialize a new scope with optional parent scope.
        
        Args:
            prior: Parent scope in the scope chain
            block: Block object associated with this scope
        """
        self.prior = prior
        self.block = block

        # Set root scope (top-level scope in the chain)
        if prior:
            self.root = prior.root
        else:
            self.root = self

        # Initialize scope storage
        self.local: dict[str, Any] = {}
        self.instance: Any | None = None
        self.graph: dict[str, Any] = {}
        self.callback: list[Any] = []
        self.instances: dict[str, Any] = {}
        self.object: dict[str, str] | None = None

    @property
    def _instance(self) -> Any | None:
        """
        Get the closest instance in the scope chain.
        
        Returns:
            The first instance found traversing up the scope chain, or None
        """
        current = self

        while current:
            if current.instance is not None:
                return current.instance
            current = current.prior

        return None

    @_instance.setter
    def _instance(self, instance: Any | None) -> None:
        """
        Set the instance for this scope.
        
        Args:
            instance: The instance object to set
        """
        self.instance = instance

    def assign(
        self,
        variable: Identifier,
        evaluation: Any,
        reassign: bool = False
    ) -> Any:
        """
        Assign a value to a variable in the scope.
        
        Args:
            variable: The identifier to assign to
            evaluation: The value to assign
            reassign: Whether this is a reassignment
            
        Returns:
            The assigned value
        """
        prefix = None

        if reassign:
            retrieved = self.retrieve(variable)
            if retrieved and hasattr(retrieved, 'object') and retrieved.object:
                prefix = retrieved.object.node

        if not prefix:
            prefix = {
                "type": "MemberExpression",
                "computed": False,
                "object": {
                    "type": "Identifier",
                    "name": "scope",
                },
                "property": {
                    "type": "Identifier",
                    "name": "local",
                },
            }

        local = Identifier(append(prefix, variable.node))

        # Python equivalent of eval assignment
        # This is a simplified version - full implementation would need proper AST evaluation
        var_name = str(variable)
        self.local[var_name] = evaluation
        return evaluation

    def retrieve(self, variable: Identifier, exact: bool = False) -> Identifier | None:
        """
        Retrieve a variable from the scope chain.
        
        Args:
            variable: The identifier to retrieve
            exact: Whether to require exact local scope match
            
        Returns:
            The identifier with resolved scope path, or None if not found
        """
        current = self

        estree = {
            "type": "Identifier",
            "name": "scope",
        }

        first = variable.first
        if not first:
            return None

        first_str = str(first)

        while current:
            if (first_str in current.graph and
                (not exact or first_str in current.local)):

                local = {
                    "type": "Identifier",
                    "name": "local",
                }
                estree = append(estree, local)
                return Identifier(append(estree, variable.node))

            prior = {
                "type": "Identifier",
                "name": "prior",
            }

            estree = append(estree, prior)
            current = current.prior

        return None

    def retrieve_instance(self, instance: str) -> Any:
        """
        Retrieve an instance by name from the scope chain.
        
        Args:
            instance: Instance name to retrieve
            
        Returns:
            The instance value, or None if not found
        """
        current = self

        while current:
            if instance in current.instances:
                return current.instances[instance]
            current = current.prior

        return None

    def retrieve_object(self) -> str | None:
        """
        Retrieve the object name from the scope chain.
        
        Returns:
            The object name, or None if no object in scope chain
        """
        current = self

        while current:
            if current.object is not None:
                return current.object.get("name")
            current = current.prior

        return None

    def retrieve_graph(self, instance: str) -> Any:
        """
        Retrieve a graph entry by instance name from the scope chain.
        
        Args:
            instance: Instance name to retrieve from graph
            
        Returns:
            The graph value, or None if not found
        """
        current = self

        while current:
            if instance in current.graph:
                return current.graph[instance]
            current = current.prior

        return None
