import $ from "./$";
import $PROPERTY from "./$PROPERTY";
import $VARIABLE from "./$VARIABLE";
import $LET from "./$LET";
import $INSTANCE from "./$INSTANCE";
import $Identifier from "../ast/Identifier";
import graph from "../../graph";
import CLASS from "../../nuc/CLASS";
import Instruction from "../../Instruction";
import {
  Expression,
  Identifier,
  MemberExpression,
  NewExpression,
  Pattern,
} from "acorn";

function build(
  kind: "VAR" | "LET" | "CONST" | null,
  left: Pattern,
  right: Expression
) {
  const statement = new $ASSIGNMENT();
  statement.knd = kind;
  statement.lft = left;
  statement.rgt = right;
  return statement;
}

class $ASSIGNMENT extends $ {
  knd!: "VAR" | "LET" | "CONST" | "PROPERTY" | null;
  lft!: Pattern;
  rgt!: Expression;
  $!: $;

  before(scope) {
    const name = new $Identifier(this.lft);

    let leftKind = this.knd;
    const reassign = !this.knd;

    if (!leftKind) {
      if (scope.retrieve(name)) {
        leftKind = "LET";
      } else {
        if (this.lft.type === "Identifier") {
          leftKind = "VAR";
        } else {
          leftKind = "PROPERTY";
        }
      }
    }

    let rightKind;

    if (!this.rgt) {
      throw SyntaxError("Missing definition");
    }

    if (this.rgt.type === "NewExpression") {
      if (
        "name" in this.rgt.callee &&
        graph.retrieve(`$${this.rgt.callee.name}`) instanceof CLASS
      ) {
        rightKind = "INSTANCE";
      } else {
        rightKind = "EXPRESSION";
      }
    } else {
      rightKind = "EXPRESSION";
    }

    switch (true) {
      case leftKind === "VAR" && rightKind === "EXPRESSION": {
        this.$ = $VARIABLE(this.lft, this.rgt);
        break;
      }
      case leftKind === "VAR" && rightKind === "INSTANCE": {
        const rgt = this.rgt as NewExpression;

        this.$ = $INSTANCE(
          rgt.callee as Identifier,
          rgt.arguments as Expression[],
          null,
          this.lft as Identifier
        );
        break;
      }
      case leftKind === "PROPERTY" && rightKind === "EXPRESSION": {
        const identifier = new $Identifier(this.lft);
        this.$ = $PROPERTY(
          identifier.object.node,
          identifier.last.node,
          this.rgt
        );
        break;
      }
      case leftKind === "PROPERTY" && rightKind === "INSTANCE": {
        const identifier = new $Identifier(this.lft);
        const rgt = this.rgt as NewExpression;

        this.$ = $INSTANCE(
          rgt.callee as Identifier,
          rgt.arguments as Expression[],
          identifier.object.node,
          identifier.last.node
        );
        break;
      }
      case ["LET", "CONST"].includes(leftKind): {
        this.$ = $LET(
          this.lft as Identifier,
          this.rgt as Identifier | MemberExpression,
          leftKind === "CONST",
          reassign
        );
        break;
      }
    }

    this.$.asg = true; // assigned
    // delete this.lft;
    // delete this.rgt;
  }

  run(scope) {
    return new Instruction(scope, this.$, null, true, null, null, null);
  }
}

export default build;
export { $ASSIGNMENT };
