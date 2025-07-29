"""Tests for Nucleoid AI runtime processor."""

import pytest
from nucleoidai.runtime import RuntimeProcessor
from nucleoidai.context import ContextManager
from nucleoidai.types import Options


class TestRuntimeProcessor:
    """Test runtime processor functionality."""
    
    def setup_method(self):
        """Set up test environment."""
        self.context_manager = ContextManager()
        self.runtime = RuntimeProcessor(self.context_manager)
    
    def test_process_simple_statement(self):
        """Test processing a simple statement."""
        options = Options(declarative=False, details=False)
        result = self.runtime.process("x = 5", options)
        assert result is not None
    
    def test_process_with_details(self):
        """Test processing with details enabled."""
        options = Options(declarative=False, details=True)
        result = self.runtime.process("y = 10", options)
        assert result is not None
        assert hasattr(result, 'string')
        assert hasattr(result, 'time')
    
    def test_process_declarative_mode(self):
        """Test processing in declarative mode."""
        options = Options(declarative=True, details=False)
        result = self.runtime.process("z = 15", options)
        assert result is not None
    
    def test_process_empty_statement(self):
        """Test processing empty statement."""
        options = Options()
        result = self.runtime.process("", options)
        assert result is None
    
    def test_process_error_handling(self):
        """Test error handling in processing."""
        options = Options(declarative=False, details=True)
        # This should handle syntax errors gracefully
        result = self.runtime.process("invalid syntax !!!", options)
        assert result is not None
        assert hasattr(result, 'error')