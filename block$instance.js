var ASSIGNMENT = require("./assignment");
var ASSIGNMENT$CLASS = require("./assignment$class");
var ASSIGNMENT$INSTANCE = require("./assignment$instance");

module.exports = class BLOCK$INSTANCE {
  constructor() {
    this.statements = [];
  }

  run() {
    let statements = [];

    this.statements.forEach(statement => {
      switch (statement.constructor) {
        case ASSIGNMENT:
        case ASSIGNMENT$CLASS: {
          let instance = new ASSIGNMENT$INSTANCE();
          instance.variable = statement.variable;
          instance.expression = statement.expression;
          instance.class = this.class;
          instance.instance = this.instance;
          statements.push(instance);
          break;
        }

        default: {
          statement.class = this.class;
          statement.instance = this.instance;
          statements.push(statement);
        }
      }
    });

    return statements;
  }

  graph() {}
};
