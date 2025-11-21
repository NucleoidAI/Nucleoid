"""
Stack module for instruction processing.
"""
from typing import Any, Dict, List, Optional


class ProcessResult:
    """Result of processing statements."""

    def __init__(self, value: Any = None, nuc: Optional[List[Any]] = None) -> None:
        """
        Initialize process result.

        Args:
            value: Result value
            nuc: List of nuc nodes
        """
        self.value: Any = value
        self.nuc: List[Any] = nuc or []

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'value': self.value,
            '$nuc': self.nuc,
        }


class Stack:
    """Stack processor for instructions."""

    def process(
        self,
        statements: List[Any],
        prior: Any,
        options: Any
    ) -> ProcessResult:
        """
        Process statements through instruction stack.

        Args:
            statements: List of statements to process
            prior: Prior scope
            options: Processing options

        Returns:
            ProcessResult with value and nuc nodes
        """
        # Lazy imports to avoid circular dependencies
        from .Scope import Scope
        from .Instruction import Instruction
        from . import graph
        from . import state
        from .__nuc__ import __NUC__
        from .nuc.BLOCK import BLOCK
        from .nuc.BREAK import BREAK
        from .nuc.EXPRESSION import EXPRESSION
        from .nuc.IF import IF
        from .nuc.NODE import NODE
        from .nuc.RETURN import RETURN

        root = Scope(prior)

        instructions: List[Instruction] = [
            Instruction(
                scope=root,
                statement=statement,
                before=True,
                run=True,
                graph=False,  # TODO Confirm graph is false
                after=False,
                derivative=False
            )
            for statement in statements
        ]

        result = ProcessResult()
        dependencies: List[Any] = []
        dependents: List[Any] = []
        priorities: List[Any] = []

        while instructions:
            instruction = instructions.pop(0)
            statement = instruction.statement

            # Case: RETURN statement
            if isinstance(statement, RETURN):
                scope = instruction.scope
                return self.process([statement.statement], scope, options)

            # Case: BREAK statement
            elif isinstance(statement, BREAK):
                inst = instructions[0] if instructions else None

                while (inst is not None and
                       statement.block is not None and
                       inst.scope.block == statement.block):
                    instructions.pop(0)
                    inst = instructions[0] if instructions else None
                    statement.block.break_flag = True

            # Case: EXPRESSION
            elif isinstance(statement, EXPRESSION):
                scope = instruction.scope

                statement.before(scope)
                evaluation = statement.run(scope, False, False)

                value = None

                if evaluation:
                    value = state.state_instance.expression(scope, evaluation)

                if value == "use declarative":
                    options.declarative = True

                if value == "use imperative":
                    options.declarative = False

                if instruction.scope == root and not instruction.derivative:
                    result.value = value

                list_next = statement.next(scope)

                if list_next:
                    new_instructions = []
                    for stmt in list_next:
                        if isinstance(stmt, Instruction):
                            new_instructions.append(stmt)
                        else:
                            new_instructions.append(
                                Instruction(
                                    scope=instruction.scope,
                                    statement=stmt,
                                    before=False,
                                    run=True,
                                    graph=False,
                                    after=False,
                                    derivative=instruction.derivative
                                )
                            )
                    instructions = new_instructions + instructions

            # Case: __NUC__ statement
            elif isinstance(statement, __NUC__):
                if instruction.before and not getattr(statement, 'prepared', False):
                    statement.before(instruction.scope)
                    statement.prepared = True

                if instruction.run:
                    next_items = statement.run(instruction.scope)
                    next_items = next_items if isinstance(next_items, list) else [next_items]
                    next_items.append(
                        Instruction(
                            scope=instruction.scope,
                            statement=statement,
                            before=False,
                            run=False,
                            graph=True,
                            after=True,
                            derivative=None
                        )
                    )

                    scope = instruction.scope

                    # Map to instructions
                    next_mapped = []
                    for stmt in next_items:
                        if isinstance(stmt, Instruction):
                            next_mapped.append(stmt)
                        else:
                            next_mapped.append(
                                Instruction(
                                    scope=scope,
                                    statement=stmt,
                                    before=True,
                                    run=True,
                                    graph=True,
                                    after=True,
                                    derivative=None
                                )
                            )

                    # Inherit properties from parent instruction
                    for stmt_inst in next_mapped:
                        stmt_inst.before = stmt_inst.before if stmt_inst.before is not None else instruction.before
                        stmt_inst.run = stmt_inst.run if stmt_inst.run is not None else instruction.run
                        stmt_inst.graph = stmt_inst.graph if stmt_inst.graph is not None else instruction.graph
                        stmt_inst.after = stmt_inst.after if stmt_inst.after is not None else instruction.after
                        stmt_inst.derivative = stmt_inst.derivative if stmt_inst.derivative is not None else instruction.derivative

                    instructions = next_mapped + instructions

                if instruction.graph:
                    statement.graph(instruction.scope)

                if instruction.after:
                    statement.after(instruction.scope)

                    if not instruction.derivative and not getattr(statement, 'asg', False):
                        if getattr(statement, 'iof', None) == "$EXPRESSION":
                            if getattr(statement, 'tkns', None) and getattr(statement.tkns, 'wrt', None):
                                result.nuc.append(statement)
                        else:
                            result.nuc.append(statement)

            # Default case
            else:
                if instruction.before:
                    statement.before(instruction.scope)

                if instruction.run:
                    scope = instruction.scope
                    run_result = statement.run(instruction.scope) or {}
                    value = run_result.get('value') if isinstance(run_result, dict) else None
                    next_items = run_result.get('next') if isinstance(run_result, dict) else None

                    if instruction.scope == root and not instruction.derivative:
                        result.value = value

                    if next_items:
                        next_items = next_items if isinstance(next_items, list) else [next_items]

                        # Map to instructions
                        next_mapped = []
                        for stmt in next_items:
                            if isinstance(stmt, Instruction):
                                next_mapped.append(stmt)
                            else:
                                next_mapped.append(
                                    Instruction(
                                        scope=scope,
                                        statement=stmt,
                                        before=True,
                                        run=True,
                                        graph=True,
                                        after=True
                                    )
                                )

                        # Inherit properties
                        for stmt_inst in next_mapped:
                            stmt_inst.before = stmt_inst.before if stmt_inst.before is not None else instruction.before
                            stmt_inst.run = stmt_inst.run if stmt_inst.run is not None else instruction.run
                            stmt_inst.graph = stmt_inst.graph if stmt_inst.graph is not None else instruction.graph

                        instructions = [i for i in next_mapped if not i.priority] + instructions
                        priorities = [i for i in next_mapped if i.priority] + priorities

                # Skip label for class handling
                skip = False
                if getattr(statement, 'type', None) == 'CLASS' and instruction.scope.prior:
                    skip = True

                if not skip:
                    if instruction.graph:
                        before_graph_result = statement.beforeGraph(instruction.scope) or {}
                        destroyed = before_graph_result.get('destroyed', False)

                        if destroyed:
                            continue

                        if isinstance(statement, NODE):
                            if graph.graph_instance.retrieve(statement.key):
                                NODE.replace(statement.key, statement)
                            else:
                                NODE.register(statement.key, statement)

                        graph_list = statement.graph(instruction.scope)

                        if options.declarative:
                            if graph_list:
                                for target in graph_list:
                                    if target.previous.get(statement.key) is not None:
                                        raise ReferenceError("Circular Dependency")

                                dependencies.extend(graph_list)

                    if hasattr(statement, 'next') and statement.next:
                        sorted_next = sorted(
                            statement.next.values(),
                            key=lambda n: getattr(n, 'sequence', 0)
                        )

                        for n in sorted_next:
                            s = instruction.scope

                            if isinstance(n, (BLOCK, IF)):
                                scope_new = Scope()
                                dependents.append(
                                    Instruction(
                                        scope=scope_new,
                                        statement=n,
                                        before=False,
                                        run=True,
                                        graph=False,
                                        after=False
                                    )
                                )
                                dependents.append(
                                    Instruction(
                                        scope=scope_new,
                                        statement=n,
                                        before=False,
                                        run=False,
                                        graph=True,
                                        after=True
                                    )
                                )
                            else:
                                dependents.append(
                                    Instruction(
                                        scope=s,
                                        statement=n,
                                        before=False,
                                        run=True,
                                        graph=False,
                                        after=False
                                    )
                                )

                    # Root scope is a scope which does not have any prior
                    if not instruction.scope.prior:
                        if not getattr(instruction.statement, 'skip', False):
                            for source in dependencies:
                                target_key = statement.key
                                NODE.direct(source.key, target_key, statement)

                        instructions.extend(dependents)
                        instructions.extend(priorities)

                        dependencies = []
                        dependents = []
                        priorities = []

                else:
                    if instruction.scope.block:
                        instruction.scope.block.stage(instruction)
                        statement.block = instruction.scope.block

        return result


# Create singleton instance
stack_instance = Stack()
