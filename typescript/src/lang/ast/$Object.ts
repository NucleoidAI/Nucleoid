import walk from "acorn-walk";
import _ from "lodash";
import $Node from "./$Node";
import $Identifier from "./$Identifier";

class $Object extends $Node {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);

      walk.simple(cloned, {
        Property(node) {
          if ($Identifier.types.includes(node.value.type)) {
            const identifier = new $Identifier(node.value);
            node.value = identifier.resolve(scope);
          }
        },
      });

      return cloned;
    } else {
      return this.node;
    }
  }
}

export default $Object;
