"""
Scope class for variable scoping and resolution.
"""
from typing import Any, Dict, List, Optional


class Scope:
    """Represents an execution scope with variable resolution."""

    def __init__(self, prior: Optional['Scope'] = None, block: Any = None) -> None:
        """
        Initialize a Scope.

        Args:
            prior: Parent scope
            block: Block this scope belongs to
        """
        self.prior: Optional['Scope'] = prior
        self.block: Any = block

        if prior:
            self.root: 'Scope' = prior.root
        else:
            self.root: 'Scope' = self

        self.local: Dict[str, Any] = {}
        self.instance: Optional[Any] = None
        self.graph: Dict[str, Any] = {}
        self.callback: List[Any] = []
        self.instances: Dict[str, Any] = {}
        self.object: Optional[Any] = None

    @property
    def dollar_instance(self) -> Optional[Any]:
        """
        Get the current instance from this scope or parent scopes.

        Returns:
            Current instance or None
        """
        index: Optional['Scope'] = self

        while index:
            instance = index.instance

            if instance:
                return instance

            index = index.prior

        return None

    @dollar_instance.setter
    def dollar_instance(self, instance: Optional[Any]) -> None:
        """
        Set the current instance.

        Args:
            instance: Instance to set
        """
        self.instance = instance

    def assign(
        self,
        variable: Any,
        evaluation: Any,
        reassign: bool = False
    ) -> Any:
        """
        Assign a value to a variable.

        Args:
            variable: Variable identifier
            evaluation: Value to assign
            reassign: Whether this is a reassignment

        Returns:
            Assigned value
        """
        from .lang.ast import __Identifier__
        from .lang.estree.estree import append

        prefix: Optional[Dict[str, Any]] = None

        if reassign:
            retrieved = self.retrieve(variable)
            if retrieved and hasattr(retrieved, 'object'):
                prefix = retrieved.object.node

        if not prefix:
            prefix = {
                'type': 'MemberExpression',
                'computed': False,
                'object': {
                    'type': 'Identifier',
                    'name': 'scope',
                },
                'property': {
                    'type': 'Identifier',
                    'name': 'local',
                },
            }

        local = __Identifier__(append(prefix, variable.node))

        # In Python, we can't use eval like JavaScript
        # We'll need to manually handle the assignment
        path = str(local).replace('scope.', 'self.')
        parts = path.split('.')

        # Navigate to the parent object
        obj = self
        for part in parts[1:-1]:  # Skip 'self' and last part
            if not hasattr(obj, part):
                setattr(obj, part, {})
            obj = getattr(obj, part)

        # Set the value
        if isinstance(obj, dict):
            obj[parts[-1]] = evaluation
        else:
            setattr(obj, parts[-1], evaluation)

        return evaluation

    def retrieve(self, variable: Any, exact: bool = False) -> Optional[Any]:
        """
        Retrieve a variable from the scope chain.

        Args:
            variable: Variable identifier
            exact: Whether to check exact value existence

        Returns:
            Variable identifier or None
        """
        from .lang.ast import __Identifier__
        from .lang.estree.estree import append

        index: Optional['Scope'] = self

        estree: Dict[str, Any] = {
            'type': 'Identifier',
            'name': 'scope',
        }

        first = variable.first if hasattr(variable, 'first') else None
        if not first:
            return None

        first_str = str(first)

        while index:
            if first_str in index.graph:
                # Check exact value if needed
                if not exact or self._check_exact(index, variable):
                    local = {
                        'type': 'Identifier',
                        'name': 'local',
                    }
                    estree = append(estree, local)
                    return __Identifier__(append(estree, variable.node))

            prior = {
                'type': 'Identifier',
                'name': 'prior',
            }

            estree = append(estree, prior)
            index = index.prior

        return None

    def _check_exact(self, index: 'Scope', variable: Any) -> bool:
        """Check if variable exists exactly in scope."""
        try:
            var_str = str(variable)
            parts = var_str.split('.')
            obj = index.local
            for part in parts:
                if isinstance(obj, dict):
                    if part not in obj:
                        return False
                    obj = obj[part]
                elif hasattr(obj, part):
                    obj = getattr(obj, part)
                else:
                    return False
            return obj is not None
        except Exception:
            return False

    def retrieve_instance(self, instance: str) -> Any:
        """
        Retrieve an instance by name.

        Args:
            instance: Instance name

        Returns:
            Instance value or None
        """
        index: Optional['Scope'] = self

        while index:
            value = index.instances.get(instance)

            if value:
                return value

            index = index.prior

        return None

    def retrieve_object(self) -> Optional[str]:
        """
        Retrieve the current object name.

        Returns:
            Object name or None
        """
        index: Optional['Scope'] = self

        while index:
            if index.object is not None:
                return index.object.name if hasattr(index.object, 'name') else str(index.object)

            index = index.prior

        return None

    def retrieve_graph(self, instance: str) -> Any:
        """
        Retrieve a value from the graph.

        Args:
            instance: Graph key

        Returns:
            Graph value or None
        """
        index: Optional['Scope'] = self

        while index:
            value = index.graph.get(instance)

            if value:
                return value

            index = index.prior

        return None
