var state = require("./state");
var Identifier = require("./identifier");
var graph = require("./graph");

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
