const Node = require("./Node");
const _ = require("lodash");
const Identifier = require("./Identifier");

class Template extends Node {
  resolve(scope) {
    if (scope) {
      const clone = _.cloneDeep(this.node);

      clone.expressions = clone.expressions.map((expression) =>
        new Identifier(expression).resolve(scope)
      );

      return clone;
    } else {
      return this.node;
    }
  }
}

module.exports = Template;
