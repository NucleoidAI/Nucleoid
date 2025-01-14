import $ from "./$";
import EXPRESSION from "../../nuc/EXPRESSION";
import $Expression from "../ast/$Expression";
import $Identifier from "../ast/$Identifier";
import REFERENCE from "../../nuc/REFERENCE";
import graph from "../../graph";
import EXPRESSION$INSTANCE from "../../nuc/EXPRESSION$INSTANCE";
import { parse } from "../estree/parser";
import { Expression } from "acorn";

function build(tokens: string | Expression | $Expression) {
  if (!(tokens instanceof $Expression)) {
    if (typeof tokens === "string") {
      const string = tokens;
      tokens = new $Expression(parse(string, false));
    } else {
      tokens = new $Expression(tokens);
    }
  }

  const statement = new $EXPRESSION();
  statement.tkns = tokens;
  return statement;
}

class $EXPRESSION extends $ {
  tkns: $Expression;

  run(scope) {
    if ($Identifier.types.includes(this.tkns.node.type)) {
      const identifier = new $Identifier(this.tkns.node);

      if (scope.retrieve(identifier)) {
        return new EXPRESSION(this.tkns);
      }

      const link = graph.retrieve(identifier);

      if (link) {
        const statement = new REFERENCE(this.tkns);
        statement.link = identifier;
        return statement;
      } else {
        return new EXPRESSION(this.tkns);
      }
    } else {
      const $instance = scope.$instance;

      if ($instance) {
        return new EXPRESSION$INSTANCE(this.tkns);
      } else {
        return new EXPRESSION(this.tkns);
      }
    }
  }
}

export default build;
export { $EXPRESSION };
