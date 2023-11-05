const state = require("../state");
const Node = require("./Node");
const Id = require("../lib/identifier");
const $EXPRESSION = require("../lang/$nuc/$EXPRESSION");
const Instruction = require("../instruction");
const LET = require("./LET");
const Scope = require("../scope");
const random = require("../lib/random");
const Evaluation = require("../lang/ast/Evaluation");
const Identifier = require("../lang/ast/Identifier");
const { append } = require("../lang/estree/estree");
const estree = require("../lang/estree/estree");

class OBJECT extends Node {
  constructor() {
    super();
    this.properties = {};
  }

  before() {
    if (this.name === undefined && this.object === undefined) {
      this.key = random(16, true);
      this.name = this.key;
    } else {
      this.key = Id.serialize(this);
    }
  }

  run(scope) {
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
    scope.object = this;

    let list = [];

    if (!this.object) {
      state.call(scope, `${this.class.list}.push`, [`state.${name}`]);
      state.assign(
        scope,
        new Identifier(`${this.class.list}["${this.name}"]`),
        new Evaluation(`state.${name}`)
      );
    }

    if (
      this.class.methods.find((method) => method.key.name === "constructor")
    ) {
      for (let i = 0; i < this.class.args.length; i++) {
        let local = new LET();
        local.name = this.class.args[i];

        if (this.args[i] !== undefined) {
          let context = $EXPRESSION(this.args[i]);
          local.value = context.statement.run();
          list.push(local);
        } else {
          let context = $EXPRESSION("undefined");
          local.value = context.statement.run();
          list.push(local);
        }
      }

      if (this.class.construct !== undefined) {
        let construct = this.class.construct;
        let instruction = new Instruction(scope, construct, false, true, false);
        list.push(instruction);
      }
    }

    for (let node in this.class.declarations) {
      let declaration = this.class.declarations[node];
      let scope = new Scope();
      scope.instances[this.class.name] = this;
      list.push(new Instruction(scope, declaration, true, true, true, true));
    }

    return { value: name, next: list };
  }

  graph() {
    if (this.object !== undefined) this.object.properties[this.name] = this;
    this.class.instances[this.key] = this;
  }

  resolve() {
    let current = this;
    let resolved = this.name.node;

    let list = [];

    while (current.object) {
      current = current.object;
      resolved = append(current.name.node, resolved);
    }

    return new Identifier(resolved);
  }
}

module.exports = OBJECT;
