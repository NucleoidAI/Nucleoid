var graph = require("./graph");
var Node = require("./node");

module.exports = class IF$INSTANCE extends Node {
  run(local) {
    if (this.condition.run(local, this.instance)) return this.true;
  }

  graph() {
    let key = Math.random() * 1000000;
    graph.node[key] = this;

    this.condition.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.variable);
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};
