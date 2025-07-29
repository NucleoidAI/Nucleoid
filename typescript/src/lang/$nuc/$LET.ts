import $ from "./$";
import graph from "../../graph";
import CLASS from "../../nuc/CLASS";
import LET from "../../nuc/LET";
import LET$CLASS from "../../nuc/LET$CLASS";
import LET$OBJECT from "../../nuc/LET$OBJECT";
import EXPRESSION from "../../nuc/EXPRESSION";
import REFERENCE from "../../nuc/REFERENCE";
import $EXPRESSION from "./$EXPRESSION";
import $Identifier from "../ast/$Identifier";
import $INSTANCE from "./$INSTANCE";
import { Expression, Identifier } from "acorn";
import NODE from "../../nuc/NODE";

function build(
  name: Identifier,
  value: Expression,
  constant: boolean = false,
  reassign: boolean = false
) {
  const statement = new $LET();
  statement.nme = name;
  statement.val = value;
  statement.cst = constant;
  statement.ras = reassign;
  return statement;
}

class $LET extends $ {
  nme!: Identifier;
  val!: Expression | NODE | $;
  cst!: boolean;
  ras!: boolean;

  before(scope) {
    if (
      "type" in this.val &&
      this.val.type === "NewExpression" &&
      "callee" in this.val &&
      "name" in this.val.callee &&
      graph.retrieve(`$${this.val.callee.name}`) instanceof CLASS
    ) {
      this.val = $INSTANCE(this.val.callee, this.val.arguments as Expression[]);
      this.val.before(scope);
    } else {
      const expression = $EXPRESSION(this.val);
      this.val = expression.run(scope);
    }
  }

  run(scope) {
    // TODO Rename this to `identifier`?
    const name = new $Identifier(this.nme);

    if (
      name.type === "MemberExpression" &&
      !scope.retrieve(name.object, true)
    ) {
      throw ReferenceError(`${name.object} is not defined`);
    }

    if (
      name.type === "MemberExpression" &&
      name?.last?.toString(scope) === "value"
    ) {
      throw TypeError("Cannot use 'value' in local");
    }

    const value = this.val;

    if (value instanceof EXPRESSION || value instanceof REFERENCE) {
      const cls = (value as EXPRESSION).tokens.find((node) => {
        const identifiers = [node.walk()].flat(Infinity);

        for (const identifier of identifiers) {
          const cls = graph.retrieve(identifier.first);
          if (cls instanceof CLASS) {
            return cls;
          }
        }
      });

      if (cls) {
        const statement = new LET$CLASS();
        statement.class = cls;
        statement.name = name;
        statement.value = value;
        statement.constant = this.cst;
        return statement;
      }

      const statement = new LET();
      statement.name = name;
      statement.value = value;
      statement.constant = this.cst;
      statement.reassign = this.ras;
      return statement;
    } else if (value.type === "$INSTANCE") {
      const object = value.run(scope);

      const statement = new LET$OBJECT();
      statement.name = name;
      statement.object = object;
      statement.constant = this.cst;

      return [object, statement];
    }
  }
}

export default build;
export { $LET };
