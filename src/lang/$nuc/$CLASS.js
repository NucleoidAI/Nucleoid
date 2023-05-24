const CLASS = require("../../nuc/CLASS");
const $ = require("./$");

function construct(name, construct, args) {
  let statement = new $CLASS();
  statement.name = `$${name}`;
  statement.construct = construct;

  if (args === undefined) {
    statement.args = [];
  } else {
    statement.args = args;
  }
  return statement;
}

class $CLASS extends $ {
  run() {
    let statement = new CLASS();
    statement.name = this.name;
    statement.args = this.args;
    statement.construct = this.construct;
    return statement;
  }
}

module.exports = construct;
