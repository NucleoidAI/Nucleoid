"""Base builder translated from `js/src/lang/$nuc/$.js`."""

from __future__ import annotations


class BaseBuilder:
    """Common lifecycle hooks for translated builder nodes."""

    def __init__(self) -> None:
        self.iof = self.__class__.__name__
        self.pre = False

    def before(self, scope: object | None = None) -> None:
        """Prepare the builder before it is compiled."""

    def run(self, scope: object | None = None) -> object:
        """Compile the builder into a runtime node."""
        raise NotImplementedError

    def graph(self, scope: object | None = None) -> object:
        """Produce graph metadata for the builder."""
        return None

    def after(self, scope: object | None = None) -> None:
        """Hook after compilation or execution."""

    @property
    def prepared(self) -> bool:
        return self.pre

    @prepared.setter
    def prepared(self, prepared: bool) -> None:
        self.pre = prepared
