"""
Instruction class for Nucleoid runtime execution.

This module defines the Instruction class which represents executable instructions
in the Nucleoid runtime, with proper typing and lifecycle hooks.
"""

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any


@dataclass
class Instruction:
    """
    Represents an executable instruction in the Nucleoid runtime.
    
    Each instruction contains scope, statement, and lifecycle hooks (before, run, graph, after)
    along with metadata about derivatives and priority.
    """

    scope: Any
    statement: Any
    before: Callable[[], None] | None
    run: Callable[[], None] | None
    graph: Callable[[], None] | None
    after: Callable[[], None] | None
    derivative: bool = True
    priority: bool = False

    def __init__(
        self,
        scope: Any,
        statement: Any,
        before: Callable[[], None] | None,
        run: Callable[[], None] | None,
        graph: Callable[[], None] | None,
        after: Callable[[], None] | None,
        derivative: bool = True,
        priority: bool = False
    ) -> None:
        """
        Initialize an Instruction with scope, statement, and lifecycle hooks.
        
        Args:
            scope: The execution scope object
            statement: The statement object to execute
            before: Pre-execution hook function
            run: Main execution function
            graph: Graph processing function
            after: Post-execution hook function
            derivative: Whether this instruction creates derivatives (default: True)
            priority: Whether this instruction has priority execution (default: False)
        """
        self.scope = scope
        self.statement = statement
        self.before = before
        self.run = run
        self.graph = graph
        self.after = after
        self.derivative = derivative
        self.priority = priority
