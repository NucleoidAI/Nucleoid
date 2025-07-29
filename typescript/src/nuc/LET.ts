import NODE from "./NODE";

class LET extends NODE {
  constructor(name: string, value: any) {
    super();
    this.name = name;
    this.value = value;
  }

  before(): void {}

  run(scope: any): { value: any } | undefined {
    const name: string = this.name;

    if (this.reassign) {
      const instance: any = scope.retrieveGraph(this.name.first);

      if (instance?.constant) {
        throw new TypeError("Assignment to constant variable.");
      }
    }

    const evaluation: any = this.value.run(scope, false, false);

    if (!evaluation) {
      return;
    }

    const value: any = scope.assign(name, evaluation, this.reassign);
    return { value };
  }

  beforeGraph(scope: any): void {
    if (!this.reassign) {
      scope.graph[this.name] = this;
    }
  }

  graph(scope: any): any {
    if (scope.block !== undefined) {
      return this.value.graph(scope);
    }
  }
}

export default LET;
