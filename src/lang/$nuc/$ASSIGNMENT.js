const $ = require("./$");
const $PROPERTY = require("./$PROPERTY");
const $VARIABLE = require("./$VARIABLE");
const $LET = require("./$LET");
const $INSTANCE = require("./$INSTANCE");
const Identifier = require("../ast/Identifier");
const graph = require("../../graph");

function build(kind, left, right) {
  let statement = new $ASSIGNMENT();
  statement.kind = kind;
  statement.left = left;
  statement.right = right;
  return statement;
}

class $ASSIGNMENT extends $ {
  run(scope) {
    const name = new Identifier(this.left);

    let leftKind = this.kind;

    if (!leftKind) {
      if (scope.retrieve(name)) {
        leftKind = "LET";
      } else {
        if (this.left.type === "Identifier") {
          leftKind = "VAR";
        } else {
          leftKind = "PROPERTY";
        }
      }
    }

    let rightKind;

    if (this.right.type === "NewExpression") {
      if (graph.retrieve(`$${this.right.callee.name}`)) {
        rightKind = "INSTANCE";
      } else {
        rightKind = "EXPRESSION";
      }
    } else {
      rightKind = "EXPRESSION";
    }

    switch (true) {
      case leftKind === "VAR" && rightKind === "EXPRESSION": {
        return $VARIABLE(this.left, this.right);
      }
      case leftKind === "VAR" && rightKind === "INSTANCE": {
        return $INSTANCE(
          this.right.callee,
          null,
          this.left,
          this.right.arguments
        );
      }
      case leftKind === "PROPERTY" && rightKind === "EXPRESSION": {
        const identifier = new Identifier(this.left);
        return $PROPERTY(
          identifier.object.node,
          identifier.last.node,
          this.right
        );
      }
      case leftKind === "PROPERTY" && rightKind === "INSTANCE": {
        const identifier = new Identifier(this.left);
        return $INSTANCE(
          this.right.callee,
          identifier.object.node,
          identifier.last.node,
          this.right.arguments
        );
      }
      case ["LET", "CONST"].includes(leftKind) && rightKind === "EXPRESSION": {
        return $LET(this.left, this.right);
      }
    }

    if (this.left.type === "Identifier") {
      if (scope.retrieve(name)) {
        return $LET(name, this.right);
      } else {
        return $VARIABLE(this.left, this.right);
      }
    } else {
      if (scope.retrieve(name)) {
        return $LET(this.left, this.right);
      } else {
        if (this.right.type === "NewExpression") {
          const cls = new Identifier(this.right.callee);

          const identifier = new Identifier(this.left);

          return $INSTANCE(
            cls,
            identifier.object.node,
            identifier.last.node,
            this.right.arguments
          );
        } else {
          const identifier = new Identifier(this.left);
          return $PROPERTY(
            identifier.object.node,
            identifier.last.node,
            this.right
          );
        }
      }
    }
  }
}

module.exports = build;
