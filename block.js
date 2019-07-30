var graph = require("./graph");
var Node = require("./node");
var Instruction = require("./instruction");
var ASSIGNMENT = require("./assignment");
var LET = require("./let");

module.exports = class BLOCK extends Node {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    let result = [];
    let lets = [];

    for (let index in this.statements) {
      let statement = this.statements[index];

      assignment: if (statement instanceof ASSIGNMENT) {
        for (let token of statement.expression.tokens) {
          if (lets[token]) {
            result.push(statement);
            break assignment;
          }
        }

        result.push(new Instruction(scope, statement));
        this.statements.splice(index, 1);
      } else if (statement instanceof LET) {
        lets[statement.variable] = true;
        result.push(statement);
      } else {
        result.push(statement);
      }
    }

    return result;
  }

  graph() {
    if (!this.statements.length) {
      return;
    }

    let statement = this.statements[0];
    let expression = statement.expression;

    let key = Date.now();
    graph.node[key] = this;

    expression.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};
