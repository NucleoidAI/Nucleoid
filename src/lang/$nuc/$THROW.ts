import $ from "./$";
import THROW from "../../nuc/THROW";
import $Expression from "../ast/$Expression";
import { Expression } from "acorn";

function build(exception: Expression) {
  const statement = new $THROW();
  statement.exc = exception;
  return statement;
}

class $THROW extends $ {
  exc!: Expression;

  run() {
    const exception = new $Expression(this.exc);
    return new THROW(exception);
  }
}

export default build;
export { $THROW };
