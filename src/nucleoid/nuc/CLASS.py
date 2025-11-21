"""
CLASS runtime node class.
"""
from typing import Any, Dict, List, Optional, Union
from .NODE import NODE


class CLASS(NODE):
    """Class declaration node."""

    def __init__(self, key: str) -> None:
        """
        Initialize a CLASS instance.

        Args:
            key: The class key identifier
        """
        super().__init__(key)
        self.methods: List[Any] = []
        self.instances: Dict[str, Any] = {}
        self.declarations: Dict[str, Any] = {}
        self.name: Any = None
        self.list: Any = None
        self.destroyed: Optional[bool] = None

    def run(self, scope: Dict[str, Any]) -> Union[NODE, List[NODE], None]:
        """
        Execute class declaration.

        Args:
            scope: The execution scope

        Returns:
            NODE, list of NODEs, or None
        """
        # Lazy imports
        from .. import graph, state
        from ..lang.Evaluation import Evaluation
        from ...__nuc__ import __ALIAS__, __EXPRESSION__

        cls = graph.retrieve(self.name)

        if cls:
            # Check if methods are equal (deep comparison)
            import json
            if json.dumps(self.methods, sort_keys=True) == json.dumps(cls.methods, sort_keys=True):
                self.destroyed = True
                return None

        state.state_instance.assign(
            scope, self.name, Evaluation(f"class {self.name}{{}}")
        )

        result_list: List[NODE] = []

        if not cls:
            state.state_instance.call(
                scope, "classes.push", [f"state.{self.name}"]
            )

            empty: Dict[str, Any] = {"type": "ArrayExpression", "elements": []}

            alias = __ALIAS__(self.name.node, self.list.node, empty)
            result_list.append(alias)

        result_list.append(
            __EXPRESSION__({"type": "Literal", "value": None, "raw": "null"})
        )

        return result_list

    def before_graph(self) -> Optional[Dict[str, bool]]:
        """
        Execute before graph operations.

        Returns:
            Dictionary with destroyed flag or None
        """
        if self.destroyed:
            return {"destroyed": True}

        from .. import graph

        cls = graph.retrieve(self.key)

        if isinstance(cls, CLASS):
            self.declarations = cls.declarations

        return None
