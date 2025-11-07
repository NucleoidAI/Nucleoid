import $Node from "./$Node";
import _ from "lodash";
import $Identifier from "./$Identifier";

class $Template extends $Node {
  resolve(scope) {
    if (scope) {
      const clone = _.cloneDeep(this.node);

      clone.expressions = clone.expressions.map((expression) =>
        new $Identifier(expression).resolve(scope)
      );

      return clone;
    } else {
      return this.node;
    }
  }
}

export default $Template;
