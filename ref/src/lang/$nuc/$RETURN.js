const $ = require("./$");
const RETURN = require("../../nuc/RETURN");

function build(statement) {
  const returnStatement = new $RETURN();
  returnStatement.stm = statement;
  return returnStatement;
}

class $RETURN extends $ {
  run() {
    return new RETURN(this.stm);
  }
}

module.exports = build;
