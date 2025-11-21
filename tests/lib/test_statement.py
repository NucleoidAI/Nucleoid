"""Tests for statement compilation utility"""

import ast

import pytest

from nucleoid.lib.statement import compile


class TestStatement:
    """Test suite for statement compilation"""

    def test_compile_simple_assignment(self):
        """Test compiling a simple assignment statement"""
        source = "x = 42"
        nodes = compile(source)

        assert len(nodes) == 1
        assert isinstance(nodes[0], ast.Assign)
        assert isinstance(nodes[0].targets[0], ast.Name)
        assert nodes[0].targets[0].id == "x"

    def test_compile_function_definition(self):
        """Test compiling a function definition"""
        source = """
def foo(x):
    return x + 1
"""
        nodes = compile(source)

        assert len(nodes) == 1
        assert isinstance(nodes[0], ast.FunctionDef)
        assert nodes[0].name == "foo"

    def test_compile_multiple_statements(self):
        """Test compiling multiple statements"""
        source = """
x = 1
y = 2
z = x + y
"""
        nodes = compile(source)

        assert len(nodes) == 3
        assert all(isinstance(node, ast.Assign) for node in nodes)

    def test_compile_expression(self):
        """Test compiling an expression"""
        source = "42 + 3"
        nodes = compile(source)

        assert len(nodes) == 1
        assert isinstance(nodes[0], ast.Expr)
        assert isinstance(nodes[0].value, ast.BinOp)

    def test_compile_class_definition(self):
        """Test compiling a class definition"""
        source = """
class MyClass:
    def __init__(self):
        self.value = 42
"""
        nodes = compile(source)

        assert len(nodes) == 1
        assert isinstance(nodes[0], ast.ClassDef)
        assert nodes[0].name == "MyClass"

    def test_compile_invalid_syntax(self):
        """Test that invalid syntax raises SyntaxError"""
        source = "x = = 42"  # Invalid syntax

        with pytest.raises(SyntaxError) as exc_info:
            compile(source)

        assert "Failed to parse source code" in str(exc_info.value)

    def test_compile_empty_string(self):
        """Test compiling an empty string"""
        source = ""
        nodes = compile(source)

        assert len(nodes) == 0

    def test_compile_comment_only(self):
        """Test compiling a comment-only string"""
        source = "# This is just a comment"
        nodes = compile(source)

        assert len(nodes) == 0

    def test_compile_if_statement(self):
        """Test compiling an if statement"""
        source = """
if x > 0:
    print("positive")
else:
    print("non-positive")
"""
        nodes = compile(source)

        assert len(nodes) == 1
        assert isinstance(nodes[0], ast.If)

    def test_compile_for_loop(self):
        """Test compiling a for loop"""
        source = """
for i in range(10):
    print(i)
"""
        nodes = compile(source)

        assert len(nodes) == 1
        assert isinstance(nodes[0], ast.For)

    def test_compile_import_statement(self):
        """Test compiling import statements"""
        source = "import os"
        nodes = compile(source)

        assert len(nodes) == 1
        assert isinstance(nodes[0], ast.Import)

    def test_compile_returns_list(self):
        """Test that compile always returns a list"""
        source = "x = 1"
        nodes = compile(source)

        assert isinstance(nodes, list)
