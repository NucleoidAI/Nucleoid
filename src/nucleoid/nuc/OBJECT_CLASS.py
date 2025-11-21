"""
OBJECT_CLASS runtime node class.
"""
from typing import Any, Dict, List
from .NODE import NODE


class OBJECT_CLASS(NODE):
    """Object class for class-based object creation."""

    def __init__(self) -> None:
        """Initialize an OBJECT_CLASS instance."""
        super().__init__()
        self.class_ref: Any = None
        self.name: str = ""
        self.type: str = "CLASS"

    def run(self, scope: Any) -> Dict[str, List[NODE]]:
        """
        Execute object class.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions
        """
        from .OBJECT_INSTANCE import OBJECT_INSTANCE
        from ..Scope import Scope
        from ..Instruction import Instruction
        from .. import graph

        # Get instances to process
        current = getattr(scope, 'dollar_instance', None)
        all_instances = [current] if current else list(self.class_ref.instances.values())

        statements: List[NODE] = []

        for inst in all_instances:
            inst_name = getattr(inst, 'name', '')
            obj_id = f"{inst_name}.{self.name}"

            statement = OBJECT_INSTANCE(obj_id)
            statement.class_ref = self.class_ref
            statement.object = graph.retrieve(inst_name)
            statement.name = self.name

            instance_scope = Scope(scope, statement)
            instance_scope.dollar_instance = inst

            instr = Instruction(
                instance_scope,
                statement,
                lambda: None,
                lambda: None,
                None,
                None
            )

            statements.append(instr)

        return {"next": statements}

    def graph(self) -> None:
        """Build dependency graph."""
        if self.class_ref:
            self.class_ref.declarations[self.key] = self
