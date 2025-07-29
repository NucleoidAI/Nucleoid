import Instruction from "../Instruction";
import NODE from "./NODE";
import PROPERTY$INSTANCE from "./PROPERTY$INSTANCE";
import Scope from "../Scope";
import _ from "lodash";

class PROPERTY$CLASS extends NODE {
  class!: {
    instances: Record<string, Record<string, unknown>>;
    declarations: Record<string, PROPERTY$CLASS>;
  };
  name!: string;
  value!: unknown;
  type = "CLASS";

  run(scope: Scope): { next: NODE[] } {
    const instance = scope.$instance as PROPERTY$INSTANCE | undefined;
    const allInstances: PROPERTY$INSTANCE[] = instance
      ? [instance]
      : (Object.values(this.class.instances) as unknown as PROPERTY$INSTANCE[]);

    const statements: NODE[] = [];

    for (const inst of allInstances) {
      const instName = (inst as any).name;
      const id = `${instName}.${this.name}`;

      const statement = new PROPERTY$INSTANCE(id) as unknown as {
        object?: Record<string, unknown>;
        name?: string;
        value?: unknown;
      };

      statement.object = inst as unknown as Record<string, unknown>;
      statement.name = this.name;
      statement.value = _.cloneDeep(this.value);

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
    // Register this property-class in declarations
    this.class.declarations[this.key] = this;
  }
}

export default PROPERTY$CLASS;
