"""Tests for Nucleoid AI core functionality."""

import pytest
from nucleoidai import start, run, register


def test_start_nucleoid():
    """Test starting the Nucleoid runtime."""
    # Should not raise any exceptions
    start()


def test_run_simple_statement():
    """Test running a simple statement."""
    start()
    result = run("x = 5")
    # Basic test - should not raise exceptions
    assert result is not None


def test_run_with_options():
    """Test running with options."""
    start()
    result = run("y = 10", {"details": True})
    assert result is not None


def test_register_function():
    """Test registering a declarative function."""
    start()
    
    def test_func():
        return "test"
    
    # Should not raise exceptions
    register(test_func)


def test_configuration():
    """Test runtime configuration."""
    config = {
        "cache": True,
        "test": True,
        "id": "test-runtime"
    }
    
    start(config)
    
    # Should start without errors
    assert True


class TestNucleoidIntegration:
    """Integration tests for Nucleoid runtime."""
    
    def setup_method(self):
        """Set up test environment."""
        start({"test": True, "cache": True})
    
    def test_multiple_statements(self):
        """Test executing multiple statements."""
        run("a = 1")
        run("b = 2") 
        result = run("c = a + b")
        # Should execute without errors
        assert result is not None
    
    def test_declarative_function(self):
        """Test declarative function execution."""
        def calc_sum(x, y):
            return x + y
        
        register(calc_sum)
        # Function should be registered without errors
        assert True