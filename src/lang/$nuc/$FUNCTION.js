const $ = require("./$");
const FUNCTION = require("../../nuc/FUNCTION");
const Identifier = require("../ast/Identifier");

function build(name, args, block) {
  let statement = new $FUNCTION();
  statement.name = name;
  statement.arguments = args;
  statement.block = block;
  return statement;
}

class $FUNCTION extends $ {
  run() {
    const name = new Identifier(this.name);
    let statement = new FUNCTION(name);
    statement.name = name;
    statement.arguments = this.arguments;
    statement.block = this.block;
    return statement;
  }
}

module.exports = build;
