"""
PROPERTY_CLASS runtime node class.
"""
from typing import Any, Dict, List
import copy
from .NODE import NODE


class PROPERTY_CLASS(NODE):
    """Property class for class-based property assignments."""

    def __init__(self) -> None:
        """Initialize a PROPERTY_CLASS instance."""
        super().__init__()
        self.class_ref: Any = None
        self.name: str = ""
        self.value: Any = None
        self.type: str = "CLASS"

    def run(self, scope: Any) -> Dict[str, List[NODE]]:
        """
        Execute property class.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions
        """
        from .PROPERTY_INSTANCE import PROPERTY_INSTANCE
        from ..Scope import Scope
        from ..Instruction import Instruction

        # Get instances to process
        instance = getattr(scope, 'dollar_instance', None)
        all_instances = [instance] if instance else list(self.class_ref.instances.values())

        statements: List[NODE] = []

        for inst in all_instances:
            inst_name = getattr(inst, 'name', '')
            prop_id = f"{inst_name}.{self.name}"

            statement = PROPERTY_INSTANCE(prop_id)
            statement.object = inst
            statement.name = self.name
            statement.value = copy.deepcopy(self.value)

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
