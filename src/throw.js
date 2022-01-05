const state = require("./state");
const Id = require("./utils/identifier");
const graph = require("./graph");

module.exports = class THROW {
  before() {}

  run(scope) {
    let exception = graph[this.exception];

    if (exception) {
      state.throw(scope, Id.serialize(exception));
    }

    throw state.run(scope, this.exception);
  }

  graph() {}
};
