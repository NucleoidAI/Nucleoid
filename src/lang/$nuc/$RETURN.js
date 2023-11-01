const $ = require("./$");
const RETURN = require("../../nuc/RETURN");

function build(statement) {
  const returnStatement = new $RETURN();
  returnStatement.statement = statement;
  return returnStatement;
}

class $RETURN extends $ {
  run() {
    return new RETURN(this.statement);
  }
}

module.exports = build;
