"""
ESTree code generator for converting AST back to code.

This module provides code generation capabilities from AST nodes,
corresponding to the TypeScript generator.ts module.
"""

from typing import Any


class ESTreeGenerator:
    """Generates code from ESTree-compatible AST nodes."""

    def generate(self, node: dict[str, Any] | Any) -> str:
        """
        Generate code string from an AST node.
        
        Args:
            node: The AST node to generate code from
            
        Returns:
            Generated code string
        """
        if not isinstance(node, dict):
            return str(node)

        node_type = node.get("type", "")

        if node_type == "Literal":
            return self._generate_literal(node)
        elif node_type == "Identifier":
            return self._generate_identifier(node)
        elif node_type == "BinaryExpression":
            return self._generate_binary_expression(node)
        elif node_type == "LogicalExpression":
            return self._generate_logical_expression(node)
        elif node_type == "UnaryExpression":
            return self._generate_unary_expression(node)
        elif node_type == "MemberExpression":
            return self._generate_member_expression(node)
        elif node_type == "CallExpression":
            return self._generate_call_expression(node)
        elif node_type == "ArrayExpression":
            return self._generate_array_expression(node)
        elif node_type == "ObjectExpression":
            return self._generate_object_expression(node)
        elif node_type == "FunctionExpression":
            return self._generate_function_expression(node)
        elif node_type == "ArrowFunctionExpression":
            return self._generate_arrow_function_expression(node)
        elif node_type == "AssignmentExpression":
            return self._generate_assignment_expression(node)
        elif node_type == "VariableDeclaration":
            return self._generate_variable_declaration(node)
        elif node_type == "ExpressionStatement":
            return self._generate_expression_statement(node)
        elif node_type == "BlockStatement":
            return self._generate_block_statement(node)
        elif node_type == "ReturnStatement":
            return self._generate_return_statement(node)
        else:
            # Default to string representation
            return str(node)

    def _generate_literal(self, node: dict[str, Any]) -> str:
        """Generate literal value."""
        value = node.get("value")
        if value is None:
            return "null"
        elif isinstance(value, bool):
            return "true" if value else "false"
        elif isinstance(value, str):
            return f'"{value}"'
        else:
            return str(value)

    def _generate_identifier(self, node: dict[str, Any]) -> str:
        """Generate identifier name."""
        return node.get("name", "unknown")

    def _generate_binary_expression(self, node: dict[str, Any]) -> str:
        """Generate binary expression."""
        left = self.generate(node.get("left", {}))
        operator = node.get("operator", "")
        right = self.generate(node.get("right", {}))
        return f"({left} {operator} {right})"

    def _generate_logical_expression(self, node: dict[str, Any]) -> str:
        """Generate logical expression."""
        left = self.generate(node.get("left", {}))
        operator = node.get("operator", "")
        right = self.generate(node.get("right", {}))
        return f"({left} {operator} {right})"

    def _generate_unary_expression(self, node: dict[str, Any]) -> str:
        """Generate unary expression."""
        operator = node.get("operator", "")
        argument = self.generate(node.get("argument", {}))
        return f"{operator}{argument}"

    def _generate_member_expression(self, node: dict[str, Any]) -> str:
        """Generate member expression."""
        object_expr = self.generate(node.get("object", {}))
        property_expr = self.generate(node.get("property", {}))
        computed = node.get("computed", False)

        if computed:
            return f"{object_expr}[{property_expr}]"
        else:
            return f"{object_expr}.{property_expr}"

    def _generate_call_expression(self, node: dict[str, Any]) -> str:
        """Generate function call expression."""
        callee = self.generate(node.get("callee", {}))
        arguments = node.get("arguments", [])
        args_str = ", ".join(self.generate(arg) for arg in arguments)
        return f"{callee}({args_str})"

    def _generate_array_expression(self, node: dict[str, Any]) -> str:
        """Generate array expression."""
        elements = node.get("elements", [])
        elements_str = ", ".join(self.generate(elem) for elem in elements if elem is not None)
        return f"[{elements_str}]"

    def _generate_object_expression(self, node: dict[str, Any]) -> str:
        """Generate object expression."""
        properties = node.get("properties", [])
        props_str = []

        for prop in properties:
            key = self.generate(prop.get("key", {}))
            value = self.generate(prop.get("value", {}))
            props_str.append(f"{key}: {value}")

        return "{" + ", ".join(props_str) + "}"

    def _generate_function_expression(self, node: dict[str, Any]) -> str:
        """Generate function expression."""
        params = node.get("params", [])
        body = node.get("body", {})

        params_str = ", ".join(self.generate(param) for param in params)
        body_str = self.generate(body)

        return f"function({params_str}) {body_str}"

    def _generate_arrow_function_expression(self, node: dict[str, Any]) -> str:
        """Generate arrow function expression."""
        params = node.get("params", [])
        body = node.get("body", {})

        params_str = ", ".join(self.generate(param) for param in params)
        body_str = self.generate(body)

        return f"({params_str}) => {body_str}"

    def _generate_assignment_expression(self, node: dict[str, Any]) -> str:
        """Generate assignment expression."""
        left = self.generate(node.get("left", {}))
        operator = node.get("operator", "=")
        right = self.generate(node.get("right", {}))
        return f"{left} {operator} {right}"

    def _generate_variable_declaration(self, node: dict[str, Any]) -> str:
        """Generate variable declaration."""
        kind = node.get("kind", "var")
        declarations = node.get("declarations", [])

        decl_strs = []
        for decl in declarations:
            id_str = self.generate(decl.get("id", {}))
            init = decl.get("init")
            if init:
                init_str = self.generate(init)
                decl_strs.append(f"{id_str} = {init_str}")
            else:
                decl_strs.append(id_str)

        return f"{kind} {', '.join(decl_strs)}"

    def _generate_expression_statement(self, node: dict[str, Any]) -> str:
        """Generate expression statement."""
        expression = node.get("expression", {})
        return self.generate(expression)

    def _generate_block_statement(self, node: dict[str, Any]) -> str:
        """Generate block statement."""
        body = node.get("body", [])
        statements = [self.generate(stmt) for stmt in body]
        return "{\n" + "\n".join(f"  {stmt};" for stmt in statements) + "\n}"

    def _generate_return_statement(self, node: dict[str, Any]) -> str:
        """Generate return statement."""
        argument = node.get("argument")
        if argument:
            return f"return {self.generate(argument)}"
        else:
            return "return"


def append(left: dict[str, Any], right: dict[str, Any]) -> dict[str, Any]:
    """
    Append a right node to a left node as a member expression.
    
    Args:
        left: The left side (object) of the member expression
        right: The right side (property) of the member expression
        
    Returns:
        A new MemberExpression node
    """
    return {
        "type": "MemberExpression",
        "computed": False,
        "object": left,
        "property": right
    }


def root(node: dict[str, Any]) -> dict[str, Any]:
    """
    Get the root object of a member expression chain.
    
    Args:
        node: The AST node to get the root from
        
    Returns:
        The root object node
    """
    current = node
    while current.get("type") == "MemberExpression" and current.get("object"):
        if current["object"].get("type") != "MemberExpression":
            break
        current = current["object"]
    return current


# Create a default generator instance
def generate(node: dict[str, Any] | Any) -> str:
    """
    Generate code from an AST node using the default generator.
    
    Args:
        node: The AST node to generate code from
        
    Returns:
        Generated code string
    """
    generator = ESTreeGenerator()
    return generator.generate(node)
