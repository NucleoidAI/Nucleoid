const $ = require("./$");
const EXPRESSION = require("../../nuc/EXPRESSION");
const Expression = require("../../Expression");
const Identifier = require("../ast/Identifier");
const REFERENCE = require("../../nuc/REFERENCE");
const graph = require("../../graph");
const EXPRESSION$INSTANCE = require("../../nuc/EXPRESSION$INSTANCE");

function build(tokens) {
  if (!(tokens instanceof Expression)) {
    if (typeof tokens === "string") {
      const { parse } = require("../estree/parser");
      const string = tokens;
      tokens = new Expression(parse(string, false));
    } else {
      tokens = new Expression(tokens);
    }
  }

  const statement = new $EXPRESSION();
  statement.tokens = tokens;
  return statement;
}

class $EXPRESSION extends $ {
  run(scope) {
    if (Identifier.types.includes(this.tokens.node.type)) {
      const identifier = new Identifier(this.tokens.node);

      if (scope.retrieve(identifier)) {
        return new EXPRESSION(this.tokens);
      }

      const link = graph.retrieve(identifier);

      if (link) {
        const statement = new REFERENCE(this.tokens);
        statement.link = identifier;
        return statement;
      } else {
        return new EXPRESSION(this.tokens);
      }
    } else {
      const $instance = scope.$instance;

      if ($instance) {
        return new EXPRESSION$INSTANCE(this.tokens);
      } else {
        return new EXPRESSION(this.tokens);
      }
    }
  }
}

module.exports = build;
