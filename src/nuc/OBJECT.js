const state = require("../state");
const Node = require("./NODE");
const Instruction = require("../instruction");
const Scope = require("../scope");
const Evaluation = require("../lang/ast/Evaluation");
const Identifier = require("../lang/ast/Identifier");
const { append } = require("../lang/estree/estree");
const estree = require("../lang/estree/estree");
const _ = require("lodash");
const $CALL = require("../lang/$nuc/$CALL");
const graph = require("../graph");
const $EXPRESSION = require("../lang/$nuc/$EXPRESSION");

class OBJECT extends Node {
  constructor(key) {
    super(key);
    this.properties = {};
  }

  run(scope) {
    scope.object = this;
    const name = this.name;

    let variable;

    if (this.object) {
      variable = new Identifier(
        estree.append(this.object.resolve().node, this.name.node)
      );
    } else {
      variable = this.name;
    }

    state.assign(
      scope,
      variable,
      new Evaluation(`new state.${this.class.name}()`),
      false
    );

    state.assign(
      scope,
      new Identifier(`${variable}.id`),
      new Evaluation(`"${name}"`)
    );

    let list = [];

    if (!this.object) {
      state.call(scope, `${this.class.list}.push`, [`state.${name}`]);
      state.assign(
        scope,
        new Identifier(`${this.class.list}["${this.name}"]`),
        new Evaluation(`state.${name}`)
      );
    }

    const constructor = graph.retrieve(`${this.class.name}.constructor`);

    if (constructor) {
      const call = $CALL(
        // TODO Accept function dynamically
        constructor.name.node,
        this.arguments.map((arg) => arg.node)
      );
      list.push(call);
    }

    for (let node in this.class.declarations) {
      let declaration = this.class.declarations[node];
      let scope = new Scope();
      scope.instances[this.class.name] = this;
      list.push(new Instruction(scope, declaration, true, true, true, true));
    }

    const expression = new Instruction(
      scope,
      $EXPRESSION(variable.node),
      true,
      true,
      false
    );
    expression.derivative = false;
    list.push(expression);

    return { next: list };
  }

  graph() {
    if (this.object !== undefined) this.object.properties[this.name] = this;
    this.class.instances[this.key] = this;
  }

  resolve() {
    let current = this;
    let resolved = this.name.node;

    while (current.object) {
      current = current.object;
      resolved = append(current.name.node, resolved);
    }

    return new Identifier(resolved);
  }
}

module.exports = OBJECT;
