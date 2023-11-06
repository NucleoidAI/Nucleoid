const AST = require("./AST");
const _ = require("lodash");
const walk = require("acorn-walk");
const Identifier = require("./Identifier");

class Function extends AST {
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
