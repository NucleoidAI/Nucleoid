import $Identifier from "../lang/ast/$Identifier";
import VARIABLE from "./VARIABLE";

class ALIAS extends VARIABLE {
  alias!: $Identifier;
  name!: $Identifier;
  value!: any;
}

export default ALIAS;
