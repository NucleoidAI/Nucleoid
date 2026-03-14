"""Variable builder translated from `js/src/lang/$nuc/$VARIABLE.js`."""

from __future__ import annotations

from nucleoid.ast import Identifier
from nucleoid.nuc import Variable

from .base import BaseBuilder
from .expression import build as build_expression


def build(name: object, value: object) -> "VariableBuilder":
    """Create a variable builder with the same shape as the JS factory."""
    statement = VariableBuilder()
    statement.nme = name
    statement.val = value
    return statement


class VariableBuilder(BaseBuilder):
    """Builds a runtime variable statement from language input."""

    def before(self, scope: object | None = None) -> None:
        expression = build_expression(self.val)
        self.val = expression.run(scope)

    def run(self, scope: object | None = None) -> Variable:
        name = Identifier(self.nme)
        statement = Variable(name)
        statement.name = name
        statement.value = self.val
        return statement
