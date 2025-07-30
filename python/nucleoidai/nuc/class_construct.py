"""CLASS construct for Nucleoid AI declarative programming."""

from typing import Any, Dict, List, Optional, Union, TYPE_CHECKING
from .node import NODE

if TYPE_CHECKING:
    from ..lang.dollar_nuc.alias import ALIAS
    from ..lang.dollar_nuc.expression import EXPRESSION

from ..lang.evaluation import Evaluation
from ..graph import get_graph_manager
from ..state import StateManager


class CLASS(NODE):
    """Represents a class declaration in the declarative runtime."""
    
    def __init__(self, key: str):
        super().__init__(key)
        self.methods: List[Any] = []
        self.instances: Dict[str, Any] = {}
        self.declarations: Dict[str, Any] = {}
        self.name: Optional[Any] = None
        self.list: Optional[Any] = None
        self.destroyed: bool = False
    
    def run(self, scope: Optional[Dict[str, Any]] = None) -> Optional[Union[NODE, List[NODE]]]:
        """Execute the class declaration."""
        if scope is None:
            scope = {}
        
        graph_manager = get_graph_manager()
        state_manager = StateManager()
        
        # Check if class already exists
        existing_class = graph_manager.get_node(str(self.name))
        
        if existing_class:
            # Compare methods to see if class has changed
            if hasattr(existing_class, 'methods') and self._methods_equal(self.methods, existing_class.methods):
                self.destroyed = True
                return None
        
        # Assign class to scope
        class_eval = Evaluation(f"class {self.name}{{}}")
        state_manager.assign(scope, self.name, class_eval)
        
        result_nodes: List[NODE] = []
        
        if not existing_class:
            # Add to classes list
            state_manager.call(scope, "classes.push", [f"state.{self.name}"])
            
            # Create empty array expression
            empty_array = {"type": "ArrayExpression", "elements": []}
            
            # Create alias for the class list (import here to avoid circular import)
            from ..lang.dollar_nuc.alias import ALIAS
            alias_node = ALIAS(self.name, self.list, empty_array)
            result_nodes.append(alias_node)
        
        # Add null expression (import here to avoid circular import)
        from ..lang.dollar_nuc.expression import EXPRESSION
        null_expr = EXPRESSION({
            "type": "Literal",
            "value": None,
            "raw": "null"
        })
        result_nodes.append(null_expr)
        
        return result_nodes
    
    def before_graph(self, scope: Optional[Dict[str, Any]] = None) -> Optional[Dict[str, Any]]:
        """Execute before graph processing."""
        if self.destroyed:
            return {"destroyed": True}
        
        graph_manager = get_graph_manager()
        existing_class = graph_manager.get_node(self.key)
        
        if existing_class and isinstance(existing_class, CLASS):
            self.declarations = existing_class.declarations
        
        return None
    
    def _methods_equal(self, methods1: List[Any], methods2: List[Any]) -> bool:
        """Compare two method lists for equality."""
        if len(methods1) != len(methods2):
            return False
        
        # Simple comparison - in a full implementation this would be more sophisticated
        for i, method in enumerate(methods1):
            if str(method) != str(methods2[i]):
                return False
        
        return True
    
    def add_method(self, method: Any) -> None:
        """Add a method to the class."""
        self.methods.append(method)
    
    def add_instance(self, instance_key: str, instance: Any) -> None:
        """Add an instance of this class."""
        self.instances[instance_key] = instance
    
    def get_instance(self, instance_key: str) -> Optional[Any]:
        """Get an instance by key."""
        return self.instances.get(instance_key)
    
    def add_declaration(self, decl_key: str, declaration: Any) -> None:
        """Add a declaration to the class."""
        self.declarations[decl_key] = declaration