"""Minimal expression builder used by translated statement builders."""

from __future__ import annotations


class ExpressionBuilder:
    """Wraps raw values behind the same `run(scope)` contract as JS builders."""

    def __init__(self, value: object) -> None:
        self.value = value

    def run(self, scope: object | None = None) -> object:
        if hasattr(self.value, "run"):
            return self.value.run(scope)

        return self.value


def build(value: object) -> object:
    """Return a builder-compatible expression wrapper."""
    if hasattr(value, "run"):
        return value

    return ExpressionBuilder(value)
