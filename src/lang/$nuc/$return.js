const $ = require("./$");
const RETURN = require("../../return");

function construct(statements) {
  const statement = new $RETURN();
  statement.statements = statements;
  return statement;
}

class $RETURN extends $ {
  run() {
    return new RETURN(this.statements);
  }
}

module.exports = construct;
