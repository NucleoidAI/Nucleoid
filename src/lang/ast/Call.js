const Identifier = require("./Identifier");
const ESTree = require("../estree/generator");
const graph = require("../../graph");
const AST = require("./AST");

class Call extends AST {
  constructor(node) {
    super(node);
    this.name = new Identifier(this.node.callee);
  }

  generate(scope) {
    const name = this.name.first().generate(scope);

    if (scope) {
      const generated = ESTree.generate(this.node);
      return graph[name] ? `state.${generated}` : generated;
    } else {
      return ESTree.generate(this.node);
    }
  }
}

module.exports = Call;
