const $ = require("./$");
const $PROPERTY = require("./$PROPERTY");
const $VARIABLE = require("./$VARIABLE");
const $LET = require("./$LET");
const $INSTANCE = require("./$INSTANCE");
const Identifier = require("../ast/Identifier");
const graph = require("../../graph");
const CLASS = require("../../nuc/CLASS");
const Instruction = require("../../instruction");

function build(kind, left, right) {
  let statement = new $ASSIGNMENT();
  statement.knd = kind;
  statement.lft = left;
  statement.rgt = right;
  return statement;
}

class $ASSIGNMENT extends $ {
  before(scope) {
    const name = new Identifier(this.lft);

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
      if (graph.retrieve(`$${this.rgt.callee.name}`) instanceof CLASS) {
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
        this.$ = $INSTANCE(this.rgt.callee, null, this.lft, this.rgt.arguments);
        break;
      }
      case leftKind === "PROPERTY" && rightKind === "EXPRESSION": {
        const identifier = new Identifier(this.lft);
        this.$ = $PROPERTY(
          identifier.object.node,
          identifier.last.node,
          this.rgt
        );
        break;
      }
      case leftKind === "PROPERTY" && rightKind === "INSTANCE": {
        const identifier = new Identifier(this.lft);
        this.$ = $INSTANCE(
          this.rgt.callee,
          identifier.object.node,
          identifier.last.node,
          this.rgt.arguments
        );
        break;
      }
      case ["LET", "CONST"].includes(leftKind): {
        this.$ = $LET(this.lft, this.rgt, leftKind === "CONST", reassign);
        break;
      }
    }

    this.$.asg = true; // assigned
    delete this.lft;
    delete this.rgt;
  }

  run(scope) {
    return new Instruction(
      scope,
      this.$,
      undefined,
      true,
      undefined,
      undefined,
      undefined
    );
  }
}

module.exports = build;
