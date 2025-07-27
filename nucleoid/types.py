"""
Type definitions for the Nucleoid runtime system.

This module contains all the type annotations, TypedDict classes, and Protocol definitions
that correspond to the TypeScript types used throughout the Nucleoid system.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Protocol, TypedDict, Union

from pydantic import BaseModel, Field


class PortConfig(TypedDict, total=False):
    """Port configuration for various services."""
    terminal: int | None
    cluster: int | None
    openapi: int | None


class DataConfig(TypedDict, total=False):
    """Data storage configuration."""
    encryption: bool | None


class Options(TypedDict, total=False):
    """Runtime options for statement execution."""
    declarative: bool | None
    details: bool | None


class Config(TypedDict, total=False):
    """Main configuration for the Nucleoid runtime."""
    path: str | None
    port: PortConfig | None
    options: Options | None
    cache: bool | None
    data: DataConfig | None
    id: str | None
    test: bool | None


class Event(BaseModel):
    """Event data structure."""
    topic: str
    data: str


class NucNode(Protocol):
    """Protocol for $nuc nodes in the execution graph."""
    type: str
    iof: str

    def before(self, scope: Any) -> None: ...
    def run(self, scope: Any) -> Any: ...
    def graph(self, scope: Any) -> None: ...
    def after(self) -> None: ...


class Result(BaseModel):
    """Execution result containing the $nuc graph and computed value."""
    nuc: list[Any] = Field(alias="$nuc", default_factory=list)
    value: Any = None


class Data(BaseModel):
    """Complete data structure for runtime execution results."""
    string: str
    declarative: bool | None = None
    result: Result
    time: int
    date: datetime
    error: bool | None = None
    events: list[Event] = Field(default_factory=list)


# AST Node types for expression processing
class BinaryExpression(TypedDict):
    """Binary expression AST node."""
    type: str  # Should be "BinaryExpression"
    left: ASTNode
    right: ASTNode
    operator: str


class LogicalExpression(TypedDict):
    """Logical expression AST node."""
    type: str  # Should be "LogicalExpression"
    left: ASTNode
    right: ASTNode
    operator: str


class UnaryExpression(TypedDict):
    """Unary expression AST node."""
    type: str  # Should be "UnaryExpression"
    argument: ASTNode
    operator: str


class BaseASTNode(TypedDict):
    """Base AST node with extensible properties."""
    type: str


# Union type for all possible AST node types
ASTNode = Union[BinaryExpression, LogicalExpression, UnaryExpression, BaseASTNode]


class BaseNode(TypedDict):
    """Base node interface for AST processing."""
    type: str


# Export commonly used type aliases
NodeDict = dict[str, Any]
ScopeDict = dict[str, Any]
