"""
Instruction class for statement execution.
"""
from typing import Any, Optional, Callable


class Instruction:
    """Represents an execution instruction."""

    def __init__(
        self,
        scope: Any,
        statement: Any,
        before: Optional[Callable[[], None]],
        run: Optional[Callable[[], None]],
        graph: Optional[Callable[[], None]],
        after: Optional[Callable[[], None]],
        derivative: bool = True,
        priority: bool = False
    ) -> None:
        """
        Initialize an Instruction.

        Args:
            scope: Execution scope
            statement: Statement to execute
            before: Before hook
            run: Run hook
            graph: Graph hook
            after: After hook
            derivative: Whether this is a derivative instruction
            priority: Whether this instruction has priority
        """
        self.scope: Any = scope
        self.statement: Any = statement
        self.before: Optional[Callable[[], None]] = before
        self.run: Optional[Callable[[], None]] = run
        self.graph: Optional[Callable[[], None]] = graph
        self.after: Optional[Callable[[], None]] = after
        self.derivative: bool = derivative
        self.priority: bool = priority
