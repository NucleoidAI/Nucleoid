import IF$INSTANCE from "./IF$INSTANCE";
import $BLOCK from "../lang/$nuc/$BLOCK";
import NODE from "./NODE";
import _ from "lodash";
import { v4 as uuid } from "uuid";
import Scope from "../Scope";
import Instruction from "../Instruction";

class IF$CLASS extends NODE {
  run(scope: Scope): { next: Instruction[] } {
    let instances: NODE[];
    let statements: Instruction[] = [];

    let instance: NODE | undefined = scope.$instance;

    if (instance) {
      instances = [instance];
    } else {
      instances = Object.values(this.class.instances);
    }

    for (let instance of instances) {
      const statement = new IF$INSTANCE(uuid());
      statement.condition = _.cloneDeep(this.condition);
      statement.true = $BLOCK(_.cloneDeep(this.true.stms));

      if (this.false?.iof === "$IF") {
        statement.false = _.cloneDeep(this.false);
      } else if (this.false?.iof === "$BLOCK") {
        statement.false = $BLOCK(_.cloneDeep(this.false.stms));
      }

      const instanceScope = new Scope(scope);
      instanceScope.$instance = instance;

      statements.push(
        new Instruction(instanceScope, statement, true, true, true, true)
      );
    }

    return { next: statements };
  }

  graph() {
    this.class.declarations[this.key] = this;
  }
}

IF$CLASS.prototype.type = "CLASS";
module.exports = IF$CLASS;
