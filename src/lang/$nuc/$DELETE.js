const $ = require("./$");
const DELETE = require("../../nuc/DELETE");
const VARIABLE = require("../../nuc/VARIABLE");
const graph = require("../../graph");
const DELETE$VARIABLE = require("../../nuc/DELETE$VARIABLE");
const DELETE$OBJECT = require("../../nuc/DELETE$OBJECT");
const OBJECT = require("../../nuc/OBJECT");
const state = require("../../state");
const $EXP = require("./$EXPRESSION");

function construct(key) {
  let statement = new $DELETE();
  statement.key = key;
  return statement;
}

class $DELETE extends $ {
  run(scope) {
    let key;

    try {
      let context = $EXP(`${this.key}.id`);
      let expression = context.statement.run();
      key = state.run(scope, expression.run(scope)) || this.key;
    } catch (error) {
      key = this.key;
    }

    if (graph[key] instanceof VARIABLE) {
      let statement = new DELETE$VARIABLE();
      statement.key = key;
      return statement;
    } else if (graph[key] instanceof OBJECT) {
      let statement = new DELETE$OBJECT();
      statement.key = key;
      return statement;
    } else {
      let statement = new DELETE();
      statement.key = key;
      return statement;
    }
  }
}

module.exports = construct;
