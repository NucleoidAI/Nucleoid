const state = require("../state");
const Local = require("../lib/local");

class LET {
  constructor(name, value) {
    this.instanceof = this.constructor.name;
    this.name = name;
    this.value = value;
  }

  before() {}

  run(scope) {
    let value = this.value.run(scope);

    const parts = this.name.split(".");
    const first = parts[0];

    if (scope.graph[first]?.instanceof === "LET$OBJECT") {
      parts[0] = scope.graph[first].object.key;
      state.assign(scope, parts.join("."), value);
    }

    let local = Local.retrieve(scope, this.name);

    if (!local) {
      local = `scope.local.${this.name}`;
    }

    let expression = `${local}=${value}`;
    return { value: state.run(scope, expression) };
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
