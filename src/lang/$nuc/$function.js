const $ = require("./$");
const FUNCTION = require("../../function");

function construct(name, args, block) {
  let statement = new $FUNCTION();
  statement.name = name;
  statement.args = args;
  statement.block = block;
  return statement;
}

class $FUNCTION extends $ {
  run() {
    let statement = new FUNCTION();
    statement.name = this.name;
    statement.args = this.args;
    statement.block = this.block;
    return statement;
  }
}

module.exports = construct;
