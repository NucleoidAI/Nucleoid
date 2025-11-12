"""Random string generation utilities"""

import random as py_random
import string

# Character sets for random string generation
ALPHANUMERIC_CHARS = string.ascii_letters + string.digits
ALPHA_CHARS = string.ascii_letters


def random(length: int = 16, alphanumeric: bool = False) -> str:
    """
    Generate a random string of specified length.

    Args:
        length: Length of the random string (default: 16)
        alphanumeric: If True, include digits; if False, only letters (default: False)

    Returns:
        A random string starting with a letter, followed by random characters
    """
    # First character is always a letter
    result = py_random.choice(ALPHA_CHARS)

    # Choose character set based on alphanumeric flag
    chars = ALPHANUMERIC_CHARS if alphanumeric else ALPHA_CHARS

    # Generate remaining characters
    for _ in range(1, length):
        result += py_random.choice(chars)

    return result
