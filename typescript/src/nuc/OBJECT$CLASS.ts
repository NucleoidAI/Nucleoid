// src/nuc/OBJECT$CLASS.ts

import Instruction from "../Instruction";
import NODE from "./NODE";
import OBJECT$INSTANCE from "./OBJECT$INSTANCE";
import Scope from "../Scope";
import { retrieve } from "../graph";

class OBJECT$CLASS extends NODE {
  class!: {
    instances: Record<string, Record<string, unknown>>;
    declarations: Record<string, OBJECT$CLASS>;
  };
  name!: string;
  type = "CLASS";

  run(scope: Scope): { next: NODE[] } {
    const current = scope.$instance as Record<string, unknown> | undefined;
    const allInstances = current
      ? [current]
      : Object.values(this.class.instances);

    const statements: NODE[] = [];

    for (const inst of allInstances) {
      const instName = (inst as any).name;
      const id = `${instName}.${this.name}`;

      const statement = new OBJECT$INSTANCE(id) as unknown as {
        class?: OBJECT$CLASS["class"];
        object?: Record<string, unknown>;
        name?: string;
      };

      statement.class = this.class;
      statement.object = retrieve(instName);
      statement.name = this.name;

      const instanceScope = new Scope(scope, statement);
      instanceScope.$instance = inst;

      const instr = new Instruction(
        instanceScope,
        statement,
        () => {},
        () => {},
        null,
        null
      ) as unknown as NODE;

      statements.push(instr);
    }

    return { next: statements };
  }

  graph(): void {
    this.class.declarations[this.key] = this;
  }
}

export default OBJECT$CLASS;
