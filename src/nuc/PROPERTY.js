const state = require("../state");
const Node = require("./NODE");
const estree = require("../lang/estree/estree");
const Identifier = require("../lang/ast/Identifier");

class PROPERTY extends Node {
  before(scope) {
    this.value.before(scope);
  }

  run(scope) {
    const evaluation = this.value.run(scope);

    if (!evaluation) {
      return;
    }

    const variable = new Identifier(
      estree.append(this.object.resolve().node, this.name.node)
    );

    const value = state.assign(scope, variable, evaluation);
    return { value };
  }

  graph(scope) {
    this.object.properties[this.name] = this;
    return this.value.graph(scope);
  }
}

module.exports = PROPERTY;
