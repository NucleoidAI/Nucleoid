const $ = require("./$");
const DELETE = require("../../nuc/DELETE");
const VARIABLE = require("../../nuc/VARIABLE");
const graph = require("../../graph");
const DELETE$VARIABLE = require("../../nuc/DELETE$VARIABLE");
const DELETE$OBJECT = require("../../nuc/DELETE$OBJECT");
const OBJECT = require("../../nuc/OBJECT");
const Identifier = require("../ast/Identifier");
const $EXPRESSION = require("./$EXPRESSION");
const state = require("../../state");

function build(key) {
  const statement = new $DELETE();
  statement.key = key;
  return statement;
}

class $DELETE extends $ {
  run(scope) {
    const identifier = new Identifier(this.key);
    let variable = graph.retrieve(identifier);

    if (!variable) {
      try {
        const $expression = $EXPRESSION(this.key);
        const expression = $expression.run(scope);
        const item = expression.run(scope);
        const { id } = state.expression(scope, { value: item });
        variable = graph.retrieve(id);
      } catch (err) {} // eslint-disable-line no-empty
    }

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
