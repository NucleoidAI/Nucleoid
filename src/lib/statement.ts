import { Expression, Node } from "acorn";
import ESTree from "../lang/estree/parser";

function compile(string: string): Node[] {
  return ESTree.parse(string);
}

export default { compile };
