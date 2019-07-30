var graph = require("./graph");
var Node = require("./node");
var BLOCK = require("./block");

module.exports = class BLOCK$INSTANCE extends BLOCK {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    scope.instance[this.class.name] = this.instance;
    return super.run(scope);
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
      token = token.replace(/.+?(?=\.)/, this.instance.variable);
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};
