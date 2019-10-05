var graph = require("./graph");
var Node = require("./node");

module.exports = class IF$INSTANCE extends Node {
  run(scope) {
    scope.instance[this.class.name] = this.instance;
    if (this.condition.run(scope, this.instance)) return this.true;
  }

  graph() {
    let key = Math.random() * 1000000;
    graph.node[key] = this;

    this.condition.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.name);
      if (graph.node[token]) Node.direct(token, key, this);
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        Node.direct(token, key, this);
      }
    });
  }
};
