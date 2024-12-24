import pytest
from centaur.Calculator import Calculator


@pytest.fixture
def calc():
    """Fixture to initialize a Calculator instance for testing."""
    return Calculator()


def test_add(calc):
    """Tests addition functionality of the Calculator."""
    assert calc.add(2, 3) == 5
    assert calc.add(-1, 1) == 0
    assert calc.add(0, 0) == 0


def test_subtract(calc):
    """Tests subtraction functionality of the Calculator."""
    assert calc.subtract(5, 3) == 2
    assert calc.subtract(0, 5) == -5
    assert calc.subtract(-5, -5) == 0


def test_multiply(calc):
    """Tests multiplication functionality of the Calculator."""
    assert calc.multiply(2, 3) == 6
    assert calc.multiply(-1, 3) == -3
    assert calc.multiply(0, 5) == 0


def test_divide(calc):
    """
    Tests division functionality of the Calculator,
    including handling division by zero.
    """
    assert calc.divide(10, 2) == 5
    assert calc.divide(-9, 3) == -3
    assert calc.divide(5, 2) == 2.5

    with pytest.raises(ValueError, match="Division by zero is not allowed"):
        calc.divide(5, 0)


def test_logger_initialization(calc):
    """Verifies that the logger is properly initialized in the Calculator."""
    assert calc.logger is not None
    assert calc.logger.name == "centaur.Calculator"


def test_result_property(calc):
    """
    Ensures the `result` attribute updates correctly after each operation.
    """
    calc.add(2, 3)
    assert calc.result == 5

    calc.subtract(5, 3)
    assert calc.result == 2

    calc.multiply(2, 3)
    assert calc.result == 6

    calc.divide(6, 3)
    assert calc.result == 2.0
