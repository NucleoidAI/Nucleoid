import { Expression, Identifier, Pattern } from "acorn";

import $ from "./$";
import $Identifier from "../ast/$Identifier";
import CLASS from "../../nuc/CLASS";
import Instruction from "../../Instruction";
import OBJECT from "../../nuc/OBJECT";
import OBJECT$CLASS from "../../nuc/OBJECT$CLASS";
import graph from "../../graph";
import random from "../../lib/random";

function build(
  cls: Identifier,
  args: Expression[] = [],
  object?: Pattern | null,
  name?: Identifier
) {
  const statement = new $INSTANCE();
  statement.cls = cls;
  statement.obj = object;
  statement.nme = name;
  statement.args = args;

  return statement;
}

class $INSTANCE extends $ {
  cls!: Identifier;
  args!: Expression[];
  obj?: Pattern | null;
  nme?: Identifier;

  before() {
    if (!this.obj && !this.nme) {
      this.nme = random(16, true);
    }
  }

  run(scope) {
    const cls = new $Identifier(`$${this.cls.name}`);
    const name = this.nme ? new $Identifier(this.nme) : null;

    // Fix: Map each arg through proper handling based on its type
    const args = this.args.map((arg) => {
      // Handle different expression types as needed
      if (arg.type === "Identifier") {
        return new $Identifier(arg);
      } else {
        // For other expression types, you might need different handling
        return new $Identifier(arg);
      }
    });

    if (!graph.retrieve(cls)) {
      throw ReferenceError(`${cls} is not defined`);
    }

    if (this.obj && name && name.toString(scope) === "value") {
      throw TypeError("Cannot use 'value' as a property");
    }

    if (this.obj) {
      const object = new $Identifier(this.obj);

      // if (scope.retrieve(object)) {
      //   const instance = new OBJECT();
      //   instance.class = this.cls;
      //   return $LET(name, instance);
      // }

      if (!graph.retrieve(object)) {
        throw ReferenceError(`${new $Identifier(this.obj)} is not defined`);
      }

      if (graph.retrieve(object.first) instanceof CLASS) {
        const statement = new OBJECT$CLASS(`${object}.${name}`);
        statement.class = graph.retrieve(cls);
        statement.name = name;
        statement.object = graph.retrieve(object);
        return [
          new Instruction(scope, statement, true, true, false, false),
          new Instruction(scope, statement, false, false, true, true),
        ];
      } else {
        const statement = new OBJECT(`${object}.${name}`);
        statement.class = graph.retrieve(cls);
        statement.name = name;
        statement.object = graph.retrieve(object);
        statement.arguments = args;
        return statement;
      }
    }

    const statement = new OBJECT(name);
    statement.class = graph.retrieve(cls);
    statement.name = name;
    statement.arguments = args;
    return statement;
  }
}

export default build;
export { $INSTANCE };
