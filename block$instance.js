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
    return this.statements;
  }

  graph() {
    let dependent = this.statements[0];

    let key = Date.now();
    graph.node[key] = this;

    dependent.value.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.name);
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};
