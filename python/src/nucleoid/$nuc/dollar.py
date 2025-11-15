# TODO Rename to NODE
from typing import Any, Optional, Union
from abc import ABC


class Dollar(ABC):
    """Base class for all dollar ($) statement types."""

    def __init__(self) -> None:
        self.type: str = self.__class__.__name__
        self.iof: str = self.__class__.__name__
        self._pre: bool = False
        self.asg: bool = False

    def before(self, scope: Any) -> None:
        """Called before the statement is executed."""
        pass

    def run(self, scope: Any) -> Optional[Union["NODE", list["NODE"], "Dollar"]]:
        """Execute the statement and return the result."""
        return None

    def graph(self, scope: Any) -> None:
        """Build the dependency graph for this statement."""
        pass

    def after(self) -> None:
        """Called after the statement is executed."""
        pass

    @property
    def prepared(self) -> bool:
        """Whether the statement has been prepared."""
        return self._pre

    @prepared.setter
    def prepared(self, value: bool) -> None:
        """Set the prepared state."""
        self._pre = value
