import BLOCK from "../../nuc/BLOCK";
import BLOCK$CLASS from "../../nuc/BLOCK$CLASS";
import $ from "./$";
import Instruction from "../../Instruction";
import Scope from "../../Scope";
import { v4 as uuid } from "uuid";
import _ from "lodash";
import LET from "../../nuc/LET";
import REFERENCE from "../../nuc/REFERENCE";
import OBJECT$CLASS from "../../nuc/OBJECT$CLASS";
import PROPERTY$CLASS from "../../nuc/PROPERTY$CLASS";

function build(statements, skip: boolean = false) {
  const statement = new $BLOCK();
  statement.stms = statements;
  statement.skp = skip;
  return statement;
}

class $BLOCK extends $ {
  stms!: $[];
  skp!: boolean;

  run(scope) {
    const test = new Scope(scope);
    test.object = scope.object;

    let $class;

    test: for (let statement of _.cloneDeep(this.stms)) {
      while (statement instanceof $) {
        if (statement.iof === "$ASSIGNMENT") {
          if (!statement.prepared) {
            statement.before(test);
            statement.prepared = true;
          }
          statement.graph(test);
          statement = statement.run(test);
          statement = statement.statement;
        }

        if (!statement.prepared) {
          statement.before(test);
          statement.prepared = true;
        }
        statement = statement.run(test);
      }

      const [result] = [statement]
        .flat(Infinity)
        .map((statement) =>
          statement instanceof Instruction ? statement.statement : statement
        );

      if (result instanceof LET && !(result.value instanceof REFERENCE)) {
        result.before(test);
        result.run(test);
        statement.beforeGraph(test);
        statement.graph(test);
        continue;
      } else if (result.type === "CLASS") {
        if (
          result instanceof PROPERTY$CLASS ||
          statement instanceof OBJECT$CLASS
        ) {
          $class = result.object;
        } else {
          $class = result.class;
        }

        break test;
      } else {
        break test;
      }
    }

    if ($class) {
      const statement = new BLOCK$CLASS(uuid());
      statement.class = $class;
      statement.statements = this.stms;
      return [
        new Instruction(scope, statement, true, true, false, false),
        new Instruction(scope, statement, false, false, true, true),
      ];
    } else {
      const statement = new BLOCK(uuid());
      statement.statements = this.stms;
      statement.skip = this.skp;

      return [
        new Instruction(scope, statement, true, true, false, false),
        new Instruction(scope, statement, false, false, true, true),
      ];
    }
  }
}

export default build;
export { $BLOCK };
