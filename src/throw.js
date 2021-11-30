const state = require("./state");
const Id = require("./utils/identifier");
const graph = require("./graph");

module.exports = class THROW {
  before() {}

  run(scope) {
    let exception = graph[this.exception];

    if (exception) {
      state.run(scope, "throw state." + Id.serialize(exception));
    }

    throw this.exception;
  }

  graph() {}
};
