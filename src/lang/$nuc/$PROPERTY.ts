import $ from "./$";
import graph from "../../graph";
import CLASS from "../../nuc/CLASS";
import PROPERTY$CLASS from "../../nuc/PROPERTY$CLASS";
import PROPERTY from "../../nuc/PROPERTY";
import OBJECT$CLASS from "../../nuc/OBJECT$CLASS";
import FUNCTION from "../../nuc/FUNCTION";
import $Identifier from "../ast/$Identifier";
import $EXPRESSION from "./$EXPRESSION";
import Instruction from "../../Instruction";
import { Expression, Identifier, Pattern } from "acorn";
import NODE from "../../nuc/NODE";

function build(object: Pattern, name: Identifier, value: Expression) {
  const statement = new $PROPERTY();
  statement.obj = object;
  statement.nme = name;
  statement.val = value;
  return statement;
}

class $PROPERTY extends $ {
  obj!: Pattern;
  nme!: Identifier;
  val!: Expression;
  $val!: NODE;

  before(scope) {
    const expression = $EXPRESSION(this.val);
    this.$val = expression.run(scope);
  }

  run(scope) {
    let object = new $Identifier(this.obj);
    const name = new $Identifier(this.nme);

    if (object.toString(scope) === "this") {
      object = scope.retrieveObject();
    }

    if (!graph.retrieve(object)) {
      throw ReferenceError(`${object} is not defined`);
    }

    if (
      name.toString(scope) === "value" &&
      !(graph[object] instanceof FUNCTION)
    ) {
      throw TypeError("Cannot use 'value' as a name");
    }

    const cls = graph.retrieve(object);
    if (cls instanceof CLASS || cls instanceof OBJECT$CLASS) {
      const statement = new PROPERTY$CLASS(`${object}.${name}`);
      statement.class = cls;
      statement.object = graph.retrieve(object);
      statement.name = name;
      statement.value = this.val;
      return [
        new Instruction(scope, statement, true, true, false, false),
        new Instruction(scope, statement, false, false, true, true),
      ];
    }

    const statement = new PROPERTY(`${object}.${name}`);
    statement.object = graph.retrieve(object);
    statement.name = name;
    statement.value = this.val;
    return new Instruction(scope, statement, true, true, true, true);
  }
}

export default build;
export { $PROPERTY };
