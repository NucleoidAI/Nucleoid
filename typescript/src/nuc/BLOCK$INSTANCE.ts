import BLOCK from "./BLOCK";
import type NODE from "./NODE";
import Scope from "../Scope";

class BLOCK$INSTANCE extends BLOCK {
  instance!: Record<string, unknown>;
  declaration!: {
    //eslint-disable-next-line @typescript-eslint/no-explicit-any
    statements: Array<any>;
  };
  break?: boolean;
  type: string;

  constructor(key: unknown) {
    super(String(key));
    this.type = "INSTANCE";
  }
  run(scope: Scope): NODE[] {
    scope.$instance = this.instance;

    if (this.break) {
      this.statements = this.declaration.statements;
      this.break = false;
    }

    return super.run(scope);
  }
}

export default BLOCK$INSTANCE;
