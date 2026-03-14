"""Identifier node used by translated language builders."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class Identifier:
    """Represents an identifier in the translated Python runtime."""

    node: object
