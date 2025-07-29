import state from "../state";
import NODE from "./NODE";

class THROW extends NODE {
  private exception: object;

  constructor(exception: object) {
    super();
    this.exception = exception;
  }

  before(): void {}

  run(scope: any): void {
    state.throw(scope, this.exception.generate(scope));
  }

  graph() {}
}

export default THROW;
