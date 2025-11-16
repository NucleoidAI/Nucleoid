"""
Evaluation class for Nucleoid language processing.
"""


class Evaluation:
    """Represents an evaluation result."""

    def __init__(self, value: str) -> None:
        """
        Initialize an Evaluation instance.

        Args:
            value: The evaluation value
        """
        self.value: str = value

    def __str__(self) -> str:
        """
        String representation of the evaluation.

        Returns:
            The evaluation value as a string
        """
        return self.value
