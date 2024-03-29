const state = require("../state");
const NODE = require("./NODE");
const Instruction = require("../Instruction");
const Scope = require("../Scope");
const Evaluation = require("../lang/Evaluation");
const Identifier = require("../lang/ast/Identifier");
const { append } = require("../lang/estree/estree");
const estree = require("../lang/estree/estree");
const $EXPRESSION = require("../lang/$nuc/$EXPRESSION");

class OBJECT extends NODE {
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

    const constructor = this.class.methods["$constructor"];

    if (constructor) {
      const $CALL = require("../lang/$nuc/$CALL");

      const call = $CALL(
        constructor,
        this.arguments.map((arg) => arg.node)
      );
      list.push(call);
    }

    for (let node in this.class.declarations) {
      const declaration = this.class.declarations[node];
      const scope = new Scope();
      scope.$instance = this;

      list.push(
        new Instruction(
          scope,
          declaration,
          true,
          true,
          false,
          false,
          true,
          true
        )
      );
      list.push(
        new Instruction(
          scope,
          declaration,
          false,
          false,
          true,
          true,
          true,
          true
        )
      );
    }

    const expression = new Instruction(
      scope,
      $EXPRESSION(variable.node),
      true,
      true,
      false,
      false
    );
    expression.derivative = false;
    list.push(expression);

    return { next: list };
  }

  graph() {
    if (this.object !== undefined) {
      this.object.properties[this.name] = this;
    }

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
