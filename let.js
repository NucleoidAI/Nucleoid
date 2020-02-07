module.exports = class LET {
  before() {}
  run(scope) {
    let value = this.value.run(scope); // eslint-disable-line no-unused-vars
    eval("scope.local." + this.name + "=value");
  }

  graph(scope) {
    scope.graph[this.name] = this;

    if (scope.block !== undefined) {
      return this.value.graph(scope);
    }
  }
};
