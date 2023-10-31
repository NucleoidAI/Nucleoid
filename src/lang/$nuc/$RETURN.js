const $ = require("./$");
const RETURN = require("../../nuc/RETURN");

function build(statements) {
  const statement = new $RETURN();
  statement.statements = statements;
  return statement;
}

class $RETURN extends $ {
  run() {
    return new RETURN(this.statements);
  }
}

module.exports = build;
