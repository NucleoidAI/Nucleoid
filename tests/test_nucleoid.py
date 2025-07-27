"""
Basic tests for Nucleoid Python implementation.

This module provides basic tests to verify the Python conversion
of the Nucleoid runtime is working correctly.
"""

import pytest
from nucleoid import nucleoid
from nucleoid.instruction import Instruction
from nucleoid.scope import Scope


def test_nucleoid_start():
    """Test that nucleoid.start() runs without error."""
    try:
        nucleoid.start({})
        assert True
    except Exception as e:
        pytest.fail(f"nucleoid.start() raised an exception: {e}")


def test_instruction_creation():
    """Test that Instruction objects can be created."""
    instruction = Instruction(
        scope={},
        statement={},
        before=None,
        run=None,
        graph=None,
        after=None
    )
    
    assert instruction.scope == {}
    assert instruction.statement == {}
    assert instruction.derivative is True
    assert instruction.priority is False


def test_scope_creation():
    """Test that Scope objects can be created."""
    scope = Scope(None, {})
    
    assert scope.prior is None
    assert scope.block == {}
    assert scope.root == scope
    assert scope.local == {}
    assert scope.instance is None


def test_scope_hierarchy():
    """Test scope hierarchy functionality."""
    parent_scope = Scope(None, {"type": "parent"})
    child_scope = Scope(parent_scope, {"type": "child"})
    
    assert child_scope.prior == parent_scope
    assert child_scope.root == parent_scope
    
    # Test instance retrieval
    parent_scope.instance = "parent_instance"
    assert child_scope._instance == "parent_instance"


if __name__ == "__main__":
    pytest.main([__file__])