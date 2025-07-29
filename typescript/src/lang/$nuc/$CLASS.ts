import CLASS from "../../nuc/CLASS";
import $ from "./$";
import { Identifier } from "acorn";
import { $FUNCTION } from "./$FUNCTION";
import $Identifier from "../ast/$Identifier";

function build(name: Identifier, methods = []) {
  const statement = new $CLASS();
  statement.nme = name;
  statement.mths = methods;
  return statement;
}

class $CLASS extends $ {
  nme!: Identifier;
  mths!: $FUNCTION[];

  run() {
    const name = new $Identifier(this.nme);

    const statement = new CLASS(`$${name}`);
    statement.name = new $Identifier(`$${name}`);
    statement.list = name;
    statement.methods = this.mths.reduce((acc, method) => {
      const identifier = method.nme;

      if (identifier.name === "constructor") {
        identifier.name = "$constructor";
      }

      acc[identifier.name] = method;
      return acc;
    }, {});

    return statement;
  }
}

export default build;
export { $CLASS };
