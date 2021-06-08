const state = require("./state");
const Identifier = require("./identifier");
const graph = require("./graph");

module.exports = class THROW {
  before() {}

  run(scope) {
    let exception = graph[this.exception];

    if (exception) {
      state.run(scope, "throw state." + Identifier.serialize(exception));
    }

    throw this.exception;
  }

  graph() {}
};
