from typing import Any, TYPE_CHECKING
import copy
from .__ import __
from uuid import uuid4

if TYPE_CHECKING:
    from typing import Optional


def build(statements: list[__], skip: bool = False) -> "__BLOCK__":
    """Build a __BLOCK__ statement."""
    statement = __BLOCK__()
    statement.stms = statements
    statement.skp = skip
    return statement


class __BLOCK__(__):
    """Represents a block of statements."""

    def __init__(self) -> None:
        super().__init__()
        self.stms: list[__] = []
        self.skp: bool = False

    def run(self, scope: Any) -> Any:
        """Execute the block of statements."""
        from ...scope import Scope
        from ...nuc.block import BLOCK
        from ...nuc.block_class import BLOCK_CLASS
        from ...instruction import Instruction
        from ...nuc.reference import REFERENCE
        from ...nuc.property_class import PROPERTY_CLASS
        from ...nuc.object_class import OBJECT_CLASS

        test = Scope(scope, {})
        test.object = scope.object
        class_obj: Any = None
        statements_copy = copy.deepcopy(self.stms)

        for statement in statements_copy:
            while isinstance(statement, __):
                if statement.iof == "__ASSIGNMENT__":
                    if not statement.prepared:
                        statement.before(test)
                        statement.prepared = True
                    statement.graph(test)
                    statement = statement.run(test)
                    statement = statement.statement

                if not statement.prepared:
                    statement.before(test)
                    statement.prepared = True
                statement = statement.run(test)

            results = [statement]
            while any(isinstance(r, list) for r in results):
                new_results = []
                for r in results:
                    if isinstance(r, list):
                        new_results.extend(r)
                    else:
                        new_results.append(r)
                results = new_results

            result = results[0] if results else statement
            from ...instruction import Instruction

            if isinstance(result, Instruction):
                result = result.statement

            if not isinstance(getattr(result, "value", None), REFERENCE):
                result.before()
                result.run(test)
                if hasattr(statement, "beforeGraph"):
                    statement.beforeGraph(test)
                statement.graph(test)
                continue
            elif result.type == "CLASS":
                if isinstance(result, PROPERTY_CLASS) or isinstance(statement, OBJECT_CLASS):
                    class_obj = result.object
                else:
                    class_obj = result.get("class") if hasattr(result, "get") else getattr(result, "class", None)
                break
            else:
                break

        if class_obj:
            statement = BLOCK_CLASS(str(uuid4()))
            setattr(statement, "class", class_obj)
            statement.statements = self.stms
            return [
                Instruction(scope, statement, None, None, None, None),
                Instruction(scope, statement, None, None, None, None),
            ]
        else:
            statement = BLOCK(str(uuid4()))
            statement.statements = self.stms
            statement.skip = self.skp
            return [
                Instruction(scope, statement, None, None, None, None),
                Instruction(scope, statement, None, None, None, None),
            ]
