var Statement = require("./statement");
var Variable = require("./variable");
var If = require("./if");

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
      return null;
    }

    if (token == "var") {
      this.statement.skip();
    }

    if (this.statement.check() == "=") {
      this.statement = new Variable(this.statement);
      this.statement.mark();
    } else if (this.statement.token == "if") {
      this.statement = new If(this.statement);
    }

    return this.statement;
  }

  extract() {
    let list = [];

    while (this.next()) {
      list.push(this.statement);
      this.statement.run();
    }

    return list;
  }
};
