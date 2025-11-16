"""
FOR runtime node class.
"""
from typing import Any, Dict, List, Optional
import copy


class FOR:
    """For loop statement node."""

    def __init__(self) -> None:
        """Initialize a FOR instance."""
        self.index: int = 0
        self.array: str = ""
        self.variable: Any = None
        self.statements: List[Any] = []

    def run(self, scope: Any) -> Optional[Dict[str, List[Any]]]:
        """
        Execute for loop.

        Args:
            scope: The execution scope

        Returns:
            Dictionary with next instructions or None

        Raises:
            TypeError: If array is not iterable
        """
        from ..lang.Evaluation import Evaluation
        from ..state import state_instance
        from .. import graph
        from ..Instruction import Instruction
        from ...__nuc__ import __LET__, __BLOCK__

        # Get the array to iterate over
        array = state_instance.expression(scope, Evaluation(f"state.{self.array}"))

        if not isinstance(array, list):
            raise TypeError(f"{self.array} is not iterable")

        if self.index < len(array):
            result_list: List[Any] = []
            item = array[self.index]
            key = item.get('id') if isinstance(item, dict) else None

            if key is not None and key in graph.graph_instance:
                obj_id = array[self.index].get('id')
                self.index += 1

                statements = [__LET__(self.variable.node, obj_id)]
                result_list.append(
                    __BLOCK__(statements + copy.deepcopy(self.statements), True)
                )
            else:
                self.index += 1

            result_list.append(Instruction(scope, self, False, True, False, False))
            return {"next": result_list}

        return None
