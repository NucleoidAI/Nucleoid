import $Node from "./$Node";
import { Literal } from "acorn";

class $Literal extends $Node {
  generate() {
    const node = this.node as Literal;
    return node.raw;
  }
}

export default $Literal;
