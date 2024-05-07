class LET {
  constructor(name, value) {
    this.name = name;
    this.value = value;
  }

  before() {}

  run(scope) {
    const name = this.name;

    if (this.reassign) {
      const instance = scope.retrieveGraph(this.name.first);

      if (instance?.constant) {
        throw TypeError("Assignment to constant variable.");
      }
    }

    const evaluation = this.value.run(scope, false, false);

    if (!evaluation) {
      return;
    }

    const value = scope.assign(name, evaluation, this.reassign);
    return { value };
  }

  beforeGraph(scope) {
    if (!this.reassign) {
      scope.graph[this.name] = this;
    }
  }

  graph(scope) {
    if (scope.block !== undefined) {
      return this.value.graph(scope);
    }
  }
}

module.exports = LET;
