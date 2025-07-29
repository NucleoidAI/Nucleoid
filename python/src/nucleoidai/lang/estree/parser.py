"""ESTree-compatible parser for Nucleoid AI."""

import ast
from typing import Any, Dict, List, Optional, Union

from ..nuc.assignment import Assignment
from ..nuc.block import Block
from ..nuc.class_declaration import ClassDeclaration
from ..nuc.delete import Delete
from ..nuc.expression import Expression
from ..nuc.for_statement import ForStatement
from ..nuc.function import Function
from ..nuc.if_statement import IfStatement
from ..nuc.instance import Instance
from ..nuc.return_statement import Return
from ..nuc.throw_statement import Throw


class ESTreeParser:
    """Parser that converts JavaScript/TypeScript-like syntax to Nucleoid constructs."""
    
    @staticmethod
    def parse(statement_string: str, map_nodes: bool = True) -> List[Any]:
        """
        Parse a statement string into Nucleoid AST nodes.
        
        Args:
            statement_string: The code to parse
            map_nodes: Whether to map nodes to Nucleoid constructs
            
        Returns:
            List of parsed nodes
        """
        try:
            # Parse Python AST first
            tree = ast.parse(statement_string)
            
            if not tree.body:
                return []
            
            if map_nodes:
                return [ESTreeParser._parse_node(node) for node in tree.body]
            else:
                # Return first expression
                if tree.body and isinstance(tree.body[0], ast.Expr):
                    return tree.body[0].value
                return tree.body[0] if tree.body else None
                
        except SyntaxError:
            # If Python parsing fails, return empty list
            return []
    
    @staticmethod
    def parse_function(statement_string: str) -> Any:
        """Parse a function definition."""
        try:
            tree = ast.parse(statement_string)
            
            if tree.body and isinstance(tree.body[0], ast.Expr):
                return tree.body[0].value
            else:
                return tree.body[0] if tree.body else None
                
        except SyntaxError:
            return None
    
    @staticmethod
    def _parse_node(node: ast.AST) -> Any:
        """Parse an individual AST node."""
        if isinstance(node, ast.Assign):
            # Variable assignment: x = 5
            if len(node.targets) == 1:
                target = node.targets[0]
                return Assignment(None, target, node.value)
        
        elif isinstance(node, ast.AugAssign):
            # Augmented assignment: x += 5
            return Assignment(None, node.target, node.value)
        
        elif isinstance(node, ast.AnnAssign):
            # Annotated assignment: x: int = 5
            return Assignment("LET", node.target, node.value)
        
        elif isinstance(node, ast.ClassDef):
            # Class definition
            methods = [ESTreeParser._parse_node(item) for item in node.body]
            return ClassDeclaration(node, methods)
        
        elif isinstance(node, ast.If):
            # If statement
            consequent = Block([ESTreeParser._parse_node(stmt) for stmt in node.body])
            alternate = None
            if node.orelse:
                alternate = Block([ESTreeParser._parse_node(stmt) for stmt in node.orelse])
            return IfStatement(node.test, consequent, alternate)
        
        elif isinstance(node, ast.For):
            # For loop
            body_statements = [ESTreeParser._parse_node(stmt) for stmt in node.body]
            return ForStatement(node.target, node.iter, body_statements)
        
        elif isinstance(node, ast.FunctionDef):
            # Function definition
            body = Block([ESTreeParser._parse_node(stmt) for stmt in node.body])
            return Function(node, node.args.args, body)
        
        elif isinstance(node, ast.Return):
            # Return statement
            return Return(node.value)
        
        elif isinstance(node, ast.Raise):
            # Raise/throw statement
            return Throw(node.exc)
        
        elif isinstance(node, ast.Delete):
            # Delete statement
            return Delete(node.targets[0] if node.targets else None)
        
        elif isinstance(node, ast.Expr):
            # Expression statement
            if isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Name):
                # Check if it's a constructor call (new instance)
                func_name = node.value.func.id
                if func_name[0].isupper():  # Convention: classes start with uppercase
                    args = node.value.args[0] if node.value.args else None
                    return Instance(node.value.func, None, None, args)
            
            return Expression(node.value)
        
        else:
            # Default: treat as expression
            return Expression(node)
    
    @staticmethod
    def _remove_locations(node: Dict[str, Any]) -> None:
        """Remove location information from parsed nodes."""
        for key in list(node.keys()):
            if key in ['start', 'end', 'lineno', 'col_offset']:
                del node[key]
            elif isinstance(node[key], dict):
                ESTreeParser._remove_locations(node[key])
            elif isinstance(node[key], list):
                for item in node[key]:
                    if isinstance(item, dict):
                        ESTreeParser._remove_locations(item)