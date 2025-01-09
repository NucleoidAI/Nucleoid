import pytest
from ..nucleoid.nucleoid import Nucleoid  # Import the Nucleoid class from the nucleoid module

@pytest.fixture
def setup():
    # Initialize the Nucleoid class
    return Nucleoid()

def test_give_variable_value(setup):
    # Use the initialized Nucleoid instance
    nucleoid = setup
    nucleoid.run("i = 1")
    assert nucleoid.run("i == 1") is True
