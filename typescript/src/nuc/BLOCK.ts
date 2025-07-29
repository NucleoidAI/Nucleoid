import { $ } from "../graph";
import Instruction from "../Instruction";
import NODE from "./NODE";
import { v4 as uuid } from "uuid";

class BLOCK extends NODE {
  public statements: any[];
  public skip: boolean;

  constructor(key) {
    super(key);
    this.statements = [];
    this.skip = false;
  }

  run(scope: any): any {
    return this.statements.map(
      (statement, index) =>
        new Instruction(scope, statement, null, null, null, null)
    );
  }
}

export default BLOCK;