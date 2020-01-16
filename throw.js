var state = require("./state"); // eslint-disable-line no-unused-vars
var Identifier = require("./identifier");
var graph = require("./graph");

module.exports = class THROW {
  before() {}

  run() {
    let exception = graph[this.exception];

    if (exception) {
      eval("throw state." + Identifier.serialize(exception));
    }

    throw this.exception;
  }

  graph() {}
};
