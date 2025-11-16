from typing import Any, Optional, Union
import copy
from .__ import __


def build(func: Any, args: list[Any]) -> "__CALL__":
    """Build a __CALL__ statement."""
    call = __CALL__()
    call.func = func
    call.args = args
    return call


class __CALL__(__):
    """Represents a function call statement."""

    def __init__(self) -> None:
        super().__init__()
        self.func: Any = None
        self.args: list[Any] = []
        self.result: Any = None

    def run(self, scope: Any) -> Optional[Union[__, "NODE", list["NODE"]]]:
        """Execute the function call."""
        from .__FUNCTION__ import __FUNCTION__
        from ..ast.__identifier__ import __Identifier__
        from ...graph import retrieve
        from .__LET__ import __let__
        from .__BLOCK__ import __block__

        block = None
        args = None

        if self.func.__class__.__name__ == "__FUNCTION__":
            func = self.func
            block = func.blk
            args = func.args
        else:
            name = __Identifier__(self.func)
            func = retrieve(name)
            block = func.block if hasattr(func, "block") else None
            args = func.arguments if hasattr(func, "arguments") else None

        if block and args:
            values = self.args
            statements = copy.deepcopy(block.stms)

            for i in range(len(args) - 1, -1, -1):
                value = values[i] if i < len(values) else {
                    "type": "Literal",
                    "value": None,
                    "raw": "null",
                }
                statements.insert(0, __let__(args[i], value))

            block_instance = __block__(statements)
            self.result = block_instance.run(scope)
            return self
        else:
            raise TypeError("This is not a function")
