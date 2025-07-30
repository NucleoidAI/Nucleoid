"""Evaluation class for Nucleoid AI language processing."""


class Evaluation:
    """Represents an evaluation expression in the declarative runtime."""
    
    def __init__(self, value: str):
        self.value = value
    
    def __str__(self) -> str:
        """Return the string representation of the evaluation."""
        return self.value
    
    def __repr__(self) -> str:
        """Return the detailed representation of the evaluation."""
        return f"Evaluation({self.value!r})"