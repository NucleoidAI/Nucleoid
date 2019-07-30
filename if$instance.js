var graph = require("./graph");
var Node = require("./node");
var IF = require("./if");

module.exports = class IF$INSTANCE extends IF {
  run(scope) {
    if (this.condition.run(scope.local, this.instance)) return this.true;
  }

  graph() {
    let key = Math.random() * 1000000;
    graph.node[key] = this;

    this.condition.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.variable);
      if (graph.node[token]) Node.direct(token, key, this);
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        Node.direct(token, key, this);
      }
    });
  }
};
