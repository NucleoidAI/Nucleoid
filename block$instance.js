var VARIABLE = require("./variable");
var VARIABLE$CLASS = require("./variable$class");
var VARIABLE$INSTANCE = require("./variable$instance");

module.exports = class BLOCK$INSTANCE {
  constructor() {
    this.statements = [];
  }

  run() {
    let statements = [];

    this.statements.forEach(statement => {
      switch (statement.constructor) {
        case VARIABLE:
        case VARIABLE$CLASS: {
          let instance = new VARIABLE$INSTANCE();
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
};
