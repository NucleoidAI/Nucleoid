import $ from "./$";
import RETURN from "../../nuc/RETURN";

function build(statement: $) {
  const returnStatement = new $RETURN();
  returnStatement.stm = statement;
  return returnStatement;
}

class $RETURN extends $ {
  stm!: $;

  run() {
    return new RETURN(this.stm);
  }
}

export default build;
export { $RETURN };
