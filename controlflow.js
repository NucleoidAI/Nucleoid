var Statement = require("./statement");
var Variable = require("./variable");

module.exports = class ControlFlow {
  constructor(string) {
    this.string = string;
  }

  next() {
    if (!this.statement) {
      this.statement = new Statement(this.string);
    } else {
      let offset = this.statement.offset;
      this.statement = new Statement(this.string);
      this.statement.offset = offset;
    }

    let token = this.statement.next();

    if (!token) {
      return false;
    }

    if (token == "var") {
      this.statement.skip();
    }

    if (this.statement.check() == "=") {
      this.statement = new Variable(this.statement);
      this.statement.mark();
    }

    return true;
  }

  run() {
    while (this.next()) {
      var result = this.configuration(this.statement);
    }

    return result;
  }

  configure(configuration) {
    this.configuration = configuration;
  }
};
