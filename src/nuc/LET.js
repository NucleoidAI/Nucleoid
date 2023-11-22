class LET {
  constructor(name, value) {
    this.instanceof = this.constructor.name;
    this.name = name;
    this.value = value;
  }

  before() {}

  run(scope) {
    const name = this.name;
    const object = this.object;

    const instance = scope.retrieveGraph(this.name.first);

    if (instance?.constant) {
      throw TypeError("Assignment to constant variable.");
    }

    const evaluation = this.value.run(scope, false, false);

    if (!evaluation) {
      return;
    }

    if (scope.graph[object]?.instanceof === "LET$OBJECT") {
      /*
      parts[0] = scope.graph[first].object.key;
      state.assign(scope, parts.join("."), value.construct());
      return { value };
      */
    } else {
      const value = scope.assign(name, evaluation);
      return { value };
    }
  }

  beforeGraph(scope) {
    scope.graph[this.name] = this;
  }

  graph(scope) {
    if (scope.block !== undefined) {
      return this.value.graph(scope);
    }
  }
}

module.exports = LET;
