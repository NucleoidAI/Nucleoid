const CLASS = require("../../nuc/CLASS");
const $ = require("./$");
const Identifier = require("../ast/Identifier");

function build(name, methods = []) {
  const statement = new $CLASS();
  statement.name = name;
  statement.methods = methods;
  return statement;
}

class $CLASS extends $ {
  run() {
    const name = new Identifier(this.name);

    const statement = new CLASS(`$${name}`);
    statement.name = new Identifier(`$${name}`);
    statement.list = name;
    statement.methods = this.methods;
    return statement;
  }
}

module.exports = build;
