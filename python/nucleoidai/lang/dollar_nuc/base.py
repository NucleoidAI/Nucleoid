"""Base class for $nuc constructs in Nucleoid AI."""

from typing import Any, Optional, Union, List, TYPE_CHECKING

if TYPE_CHECKING:
    from ...nuc.node import NODE


class DollarBase:
    """Base class for all $ (dollar) constructs in the Nucleoid language."""
    
    def __init__(self):
        self.type = self.__class__.__name__
        self.iof = self.__class__.__name__
        self._pre = False
        self.asg = False
    
    def before(self, scope: Optional[Any] = None) -> None:
        """Execute before processing."""
        pass
    
    def run(self, scope: Optional[Any] = None) -> Optional[Union['NODE', List['NODE'], 'DollarBase']]:
        """Execute the main logic."""
        return None
    
    def graph(self, scope: Optional[Any] = None) -> None:
        """Execute graph processing."""
        pass
    
    def after(self) -> None:
        """Execute after processing."""
        pass
    
    @property
    def prepared(self) -> bool:
        """Check if the construct is prepared."""
        return self._pre
    
    @prepared.setter
    def prepared(self, prepared: bool) -> None:
        """Set the prepared state."""
        self._pre = prepared
    
    def __str__(self) -> str:
        """String representation."""
        return f"{self.__class__.__name__}()"
    
    def __repr__(self) -> str:
        """Detailed representation."""
        return f"{self.__class__.__name__}(type={self.type}, prepared={self.prepared})"