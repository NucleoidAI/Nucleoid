import $ from "./$";
import DELETE from "../../nuc/DELETE";
import VARIABLE from "../../nuc/VARIABLE";
import graph from "../../graph";
import DELETE$VARIABLE from "../../nuc/DELETE$VARIABLE";
import DELETE$OBJECT from "../../nuc/DELETE$OBJECT";
import OBJECT from "../../nuc/OBJECT";
import Identifier from "../ast/Identifier";
import $EXPRESSION from "./$EXPRESSION";
import state from "../../state";
import { Expression } from "acorn";

function build(key: Expression) {
  const statement = new $DELETE();
  statement.key = key;
  return statement;
}

class $DELETE extends $ {
  key!: Expression;

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
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
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

export default build;
export { $DELETE };
