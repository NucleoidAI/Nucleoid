const state = require("../state");

class THROW {
  constructor(exception) {
    this.exception = exception;
  }

  before() {}

  run(scope) {
    state.throw(scope, this.exception.generate(scope));
  }

  graph() {}
}

module.exports = THROW;
