const Node = require("./Node");
const _ = require("lodash");
const walk = require("acorn-walk");
const Identifier = require("./Identifier");

class Function extends Node {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);

      walk.simple(cloned.body, {
        Identifier(node) {
          const identifier = new Identifier(node);
          return identifier.resolve(scope);
        },
      });

      return cloned;
    } else {
      return this.node;
    }
  }
}

module.exports = Function;
