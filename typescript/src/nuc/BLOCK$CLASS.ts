// src/nuc/BLOCK$CLASS.ts

import BLOCK$INSTANCE from "./BLOCK$INSTANCE";
import Instruction from "../Instruction";
import NODE from "./NODE";
import Scope from "../Scope";
import { v4 as uuid } from "uuid";

class BLOCK$CLASS extends NODE {
  statements: any[];
  class!: {
    instances: Record<string, Record<string, unknown>>;
    declarations: Record<string, BLOCK$CLASS>;
  };
  id!: string;
  type: string;

  constructor(key: unknown) {
    super(typeof key === "string" ? key : String(key));
    this.statements = [];
    this.type = "CLASS";
  }

  run(scope: Scope): NODE[] | null {
    const instances: Record<string, unknown>[] = scope.$instance
      ? [scope.$instance as Record<string, unknown>]
      : Object.values(this.class.instances);

    const instructions: NODE[] = [];

    for (const instance of instances) {
      const statement = new BLOCK$INSTANCE(uuid()) as unknown as {
        id: string;
        class?: {
          instances: Record<string, Record<string, unknown>>;
          declarations: Record<string, BLOCK$CLASS>;
        };
        instance?: Record<string, unknown>;
        statements?: any[];
        declaration?: BLOCK$CLASS;
      };

      statement.id = uuid();
      statement.class = this.class;
      statement.instance = instance;
      statement.statements = this.statements;
      statement.declaration = this;

      const parentScope = (scope as any).block ? scope : null;
      const instanceScope = new Scope(parentScope, statement);
      instanceScope.$instance = this;

      instructions.push(
        new Instruction(
          instanceScope,
          statement,
          () => {},
          () => {},
          null,
          null
        ) as unknown as NODE
      );

      instructions.push(
        new Instruction(
          instanceScope,
          statement,
          () => {},
          () => {},
          null,
          null
        ) as unknown as NODE
      );
    }

    return instructions.length > 0 ? instructions : null;
  }

  graph(): NODE[] | null {
    this.class.declarations[this.id] = this;
    return null;
  }
}

export default BLOCK$CLASS;

