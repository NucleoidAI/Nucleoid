const AST = require("./AST");

class Operator extends AST {
  static get types() {
    return ["BinaryExpression", "LogicalExpression"];
  }
}

module.exports = Operator;
