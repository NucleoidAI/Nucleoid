var graph = require("./graph");
var Node = require("./node");

module.exports = class BLOCK$INSTANCE extends Node {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    scope.instance[this.class.name] = this.instance;
    return this.statements;
  }

  graph() {
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
