const CLASS = require("../../nuc/CLASS");
const $ = require("./$");

function build(name, methods = []) {
  const statement = new $CLASS();
  statement.name = name;
  statement.methods = methods;
  return statement;
}

class $CLASS extends $ {
  run() {
    const statement = new CLASS();
    const name = this.name.resolve();
    statement.name = `$${name}`;
    statement.list = name;
    statement.methods = this.methods;
    return statement;
  }
}

module.exports = build;
