"""
OBJECT runtime node class.
"""
from typing import Any, Dict, List
from .NODE import NODE


class OBJECT(NODE):
    """Object creation node."""

    def __init__(self, key: Any) -> None:
        """
        Initialize an OBJECT instance.

        Args:
            key: The object key
        """
        super().__init__(key)
        self.properties: Dict[str, Any] = {}
        self.name: Any = None
        self.object: Any = None
        self.class_ref: Any = None
        self.arguments: List[Any] = []

    def run(self, scope: Any) -> Dict[str, List[Any]]:
        """
        Execute object creation.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions
        """
        from ..lang.ast import __Identifier__
        from ..lang.Evaluation import Evaluation
        from ..lang.estree.estree import append
        from ..state import state_instance
        from ..Instruction import Instruction
        from ..Scope import Scope
        from ...__nuc__ import __EXPRESSION__, __CALL__

        scope.object = self
        name = self.name

        # Determine variable path
        if self.object:
            variable = __Identifier__(
                append(self.object.resolve().node, self.name.node)
            )
        else:
            variable = self.name

        # Assign new instance
        state_instance.assign(
            scope,
            variable,
            Evaluation(f"new state.{self.class_ref.name}()"),
            False
        )

        # Assign ID
        state_instance.assign(
            scope,
            __Identifier__(f"{variable}.id"),
            Evaluation(f'"{name}"')
        )

        result_list: List[Any] = []

        # Handle non-nested objects
        if not self.object:
            state_instance.call(scope, f"{self.class_ref.list}.push", [f"state.{name}"])
            state_instance.assign(
                scope,
                __Identifier__(f'{self.class_ref.list}["{self.name}"]'),
                Evaluation(f"state.{name}")
            )

        # Call constructor if exists
        constructor = self.class_ref.methods.get("$constructor") if self.class_ref else None
        if constructor:
            call = __CALL__(
                constructor,
                [arg.node for arg in self.arguments]
            )
            result_list.append(call)

        # Process class declarations
        if self.class_ref:
            for node_key in self.class_ref.declarations:
                declaration = self.class_ref.declarations[node_key]
                decl_scope = Scope()
                decl_scope.dollar_instance = self

                result_list.append(
                    Instruction(
                        decl_scope,
                        declaration,
                        True,
                        True,
                        False,
                        False,
                        True,
                        True
                    )
                )
                result_list.append(
                    Instruction(
                        decl_scope,
                        declaration,
                        False,
                        False,
                        True,
                        True,
                        True,
                        True
                    )
                )

        # Add expression
        expression = Instruction(
            scope,
            __EXPRESSION__(variable.node),
            True,
            True,
            False,
            False
        )
        expression.derivative = False
        result_list.append(expression)

        return {"next": result_list}

    def graph(self) -> None:
        """Build dependency graph."""
        if self.object is not None:
            self.object.properties[str(self.name)] = self

        if self.class_ref:
            self.class_ref.instances[self.key] = self

    def resolve(self) -> Any:
        """
        Resolve object path.

        Returns:
            Identifier with full path
        """
        from ..lang.ast import __Identifier__
        from ..lang.estree.estree import append

        current = self
        resolved = self.name.node

        while current.object:
            current = current.object
            resolved = append(current.name.node, resolved)

        return __Identifier__(resolved)
