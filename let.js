module.exports = class LET {
  prepare() {}
  run(scope) {
    let value = this.value.run(scope); // eslint-disable-line no-unused-vars
    eval("scope.local." + this.name + "=value");
  }

  graph(scope) {
    scope.graph[this.name] = this;
    return this.value.graph(scope);
  }
};
