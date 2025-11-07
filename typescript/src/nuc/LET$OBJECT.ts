const LET = require("./LET");
const Evaluation = require("../lang/Evaluation");

class LET$OBJECT extends LET {
  before(): void {}

  run(scope: any): { value: string } {
    const evaluation: Evaluation = new Evaluation(this.object.key);
    const value: string = scope.assign(this.name, `state.${evaluation}`);
    return { value };
  }

  graph(): void {}
}

export default LET$OBJECT;
