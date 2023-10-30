const Identifier = require("./Identifier");
const ESTree = require("../estree/generator");
const graph = require("../../graph");

class Call {
  constructor(node) {
    this.node = node;
    this.name = new Identifier(this.node.callee);
  }

  resolve(path = false) {
    const name = this.name.object.resolve();

    if (path) {
      const generated = ESTree.generate(this.node);
      return graph[name] ? `state.${generated}` : generated;
    } else {
      return ESTree.generate(this.node);
    }
  }
}

module.exports = Call;
