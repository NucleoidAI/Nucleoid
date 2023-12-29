const $ = require("./$");
const FUNCTION = require("../../nuc/FUNCTION");
const Identifier = require("../ast/Identifier");

function build(name, args, block) {
  let statement = new $FUNCTION();
  statement.nme = name;
  statement.args = args;
  statement.blk = block;
  return statement;
}

class $FUNCTION extends $ {
  run() {
    const name = new Identifier(this.nme);
    let statement = new FUNCTION(name);
    statement.name = name;
    statement.arguments = this.args;
    statement.block = this.blk;
    return statement;
  }
}

module.exports = build;
