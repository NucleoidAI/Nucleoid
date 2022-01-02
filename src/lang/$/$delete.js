const $ = require("./$");
const DELETE = require("../../delete");
const VARIABLE = require("../../variable");
const graph = require("../../graph");
const DELETE$VARIABLE = require("../../delete$variable");
const DELETE$OBJECT = require("../../delete$object");
const OBJECT = require("../../object");
const state = require("../../state");

module.exports = function (key) {
  let statement = new $DELETE();
  statement.key = key;
  return statement;
};

class $DELETE extends $ {
  run(scope) {
    let key;

    try {
      key = state.run(scope, `state.${this.key}.id`) || this.key;
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
