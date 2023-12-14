const state = require("../state");
const Node = require("./NODE");
const estree = require("../lang/estree/estree");
const Identifier = require("../lang/ast/Identifier");
const REFERENCE = require("./REFERENCE");
const graph = require("../graph");

class PROPERTY extends Node {
  before(scope) {
    this.value.before(scope);
  }

  run(scope) {
    const evaluation = this.value.run(scope);

    const object =
      this.object.value instanceof REFERENCE
        ? graph.retrieve(this.object.value.link)
        : this.object;

    // TODO Rename `variable`
    const variable = new Identifier(
      estree.append(object.resolve().node, this.name.node)
    );

    if (!evaluation) {
      state.delete(scope, variable);
      return;
    }

    const value = state.assign(scope, variable, evaluation);
    return { value };
  }

  graph(scope) {
    const object =
      this.object.value instanceof REFERENCE
        ? graph.retrieve(this.object.value.link)
        : this.object;

    object.properties[this.name] = this;
    return this.value.graph(scope);
  }
}

module.exports = PROPERTY;
