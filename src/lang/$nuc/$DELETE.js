const $ = require("./$");
const DELETE = require("../../nuc/DELETE");
const VARIABLE = require("../../nuc/VARIABLE");
const graph = require("../../graph");
const DELETE$VARIABLE = require("../../nuc/DELETE$VARIABLE");
const DELETE$OBJECT = require("../../nuc/DELETE$OBJECT");
const OBJECT = require("../../nuc/OBJECT");
const Identifier = require("../ast/Identifier");

function build(key) {
  const statement = new $DELETE();
  statement.key = key;
  return statement;
}

class $DELETE extends $ {
  run() {
    const identifier = new Identifier(this.key);
    const variable = graph.retrieve(identifier);

    if (variable instanceof VARIABLE) {
      const statement = new DELETE$VARIABLE();
      statement.variable = variable;
      return statement;
    } else if (variable instanceof OBJECT) {
      const statement = new DELETE$OBJECT();
      statement.variable = variable;
      return statement;
    } else {
      // TODO Rename this for property
      const statement = new DELETE();
      statement.variable = variable;
      return statement;
    }
  }
}

module.exports = build;
