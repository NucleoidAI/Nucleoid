import state from "../state";
import NODE from "./NODE";

class VARIABLE extends NODE {
  private value: NODE;

  constructor(value: NODE) {
    super();
    this.value = value;
  }

  before(scope: object): void {
    this.value.before(scope);
  }

  run(scope: object): { value: any } {
    const evaluation: any = this.value.run(scope);
    const value: any = state.assign(scope, this.name, evaluation);

    return { value };
  }

  graph(scope) {
    return this.value.graph(scope);
  }
}

module.exports = VARIABLE;
