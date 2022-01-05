const state = require("./state");
const Node = require("./node");
const Id = require("./utils/identifier");
const $EXP = require("./lang/$/$expression");
const Instruction = require("./instruction");
const LET = require("./let");
const Scope = require("./scope");
const _ = require("lodash");

module.exports = class OBJECT extends Node {
  constructor() {
    super();
    this.properties = {};
  }

  before() {
    if (this.name === undefined && this.object === undefined) {
      this.key = _.camelCase(this.class.name) + this.class.sequence++;
      this.name = this.key;
    } else {
      this.key = Id.serialize(this);
    }
  }

  run(scope) {
    let name = this.key;

    state.assign(scope, name, `new state.${this.class.name}()`);
    scope.object = this;

    let list = [];

    for (let i = 0; i < this.class.args.length; i++) {
      let local = new LET();
      local.name = this.class.args[i];

      if (this.args[i] !== undefined) {
        let context = $EXP(this.args[i]);
        local.value = context.statement.run();
        list.push(local);
      } else {
        let context = $EXP("undefined");
        local.value = context.statement.run();
        list.push(local);
      }
    }

    if (this.class.construct !== undefined) {
      let construct = this.class.construct;
      let instruction = new Instruction(scope, construct, false, true, false);
      list.push(instruction);
    }

    for (let node in this.class.declarations) {
      let declaration = this.class.declarations[node];
      let scope = new Scope();
      scope.instances[this.class.name] = this;
      list.push(new Instruction(scope, declaration, true, true, true, true));
    }

    if (this.object === undefined) {
      let context = $EXP(`${this.class.name.substring(1)}.push(${this.name})`);
      list.push(context.statement);

      context = $EXP(
        `${this.class.name.substring(1)}["${this.name}"]=${this.name}`
      );
      list.push(context.statement);

      state.run(scope, `state.${name}.id="${name}"`);

      context = $EXP(this.name);
      list.push(
        new Instruction(
          scope,
          context.statement,
          true,
          true,
          false,
          null,
          false
        )
      );
    }

    return { value: name, next: list };
  }

  graph() {
    if (this.object !== undefined) this.object.properties[this.name] = this;
    this.class.instances[this.key] = this;
  }
};
