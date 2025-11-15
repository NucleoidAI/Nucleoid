from typing import Any, Optional, Union
import copy
from .dollar import Dollar
# from .dollar_block import DollarBLOCK
# from .dollar_function import DollarFUNCTION
# from ..ast.identifier import Identifier as DollarIdentifier
# from .dollar_let import dollar_let
# from ...nuc.node import NODE
# from ...graph import retrieve


def build(func: Any, args: list[Any]) -> "DollarCALL":
    """Build a DollarCALL statement."""
    call = DollarCALL()
    call.func = func
    call.args = args
    return call


class DollarCALL(Dollar):
    """Represents a function call statement."""

    def __init__(self) -> None:
        super().__init__()
        self.func: Any = None
        self.args: list[Any] = []
        self.result: Any = None

    def run(self, scope: Any) -> Optional[Union[Dollar, "NODE", list["NODE"]]]:
        """Execute the function call."""
        # Imports moved here to avoid circular dependencies
        from .dollar_function import DollarFUNCTION
        from ..ast.identifier import Identifier as DollarIdentifier
        from ...graph import retrieve
        from .dollar_let import dollar_let
        from .dollar_block import dollar_block

        block = None
        args = None

        if self.func.__class__.__name__ == "DollarFUNCTION":
            func = self.func
            block = func.blk
            args = func.args
        else:
            name = DollarIdentifier(self.func)
            func = retrieve(name)
            block = func.block if hasattr(func, "block") else None
            args = func.arguments if hasattr(func, "arguments") else None

        if block and args:
            values = self.args

            statements = copy.deepcopy(block.stms)

            # Insert parameter assignments at the beginning (in reverse order)
            for i in range(len(args) - 1, -1, -1):
                # Create a literal null value if no argument provided
                value = values[i] if i < len(values) else {
                    "type": "Literal",
                    "value": None,
                    "raw": "null",
                }
                statements.insert(0, dollar_let(args[i], value))

            block_instance = dollar_block(statements)
            self.result = block_instance.run(scope)
            return self
        else:
            raise TypeError("This is not a function")
