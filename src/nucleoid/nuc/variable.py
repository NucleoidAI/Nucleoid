"""Variable runtime node translated from the JavaScript implementation."""

from __future__ import annotations

from dataclasses import dataclass, field

from .node import Node


@dataclass
class Variable(Node):
    """Assigns a resolved value to a named slot in the provided scope."""

    key: object
    name: object | None = None
    value: object | None = None
    next: dict[str, object] = field(default_factory=dict)
    previous: dict[str, object] = field(default_factory=dict)

    def before(self, scope: object | None = None) -> None:
        if hasattr(self.value, "before"):
            self.value.before(scope)

    def run(self, scope: object | None = None) -> dict[str, object]:
        if hasattr(self.value, "run"):
            evaluation = self.value.run(scope)
        else:
            evaluation = self.value

        if scope is None or not hasattr(scope, "assign"):
            raise TypeError("scope must provide an assign(name, value) method")

        value = scope.assign(self.name, evaluation)
        return {"value": value}

    def graph(self, scope: object | None = None) -> object:
        if hasattr(self.value, "graph"):
            return self.value.graph(scope)

        return None
