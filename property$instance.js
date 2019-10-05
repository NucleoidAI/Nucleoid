var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");

module.exports = class PROPERTY$INSTANCE extends Node {
  run(scope) {
    let instance = state[this.instance.name];
    instance[this.name] = this.value.run(scope, this.instance);
  }

  graph() {
    let key = this.instance.name + "." + this.name;
    graph.node[key] = this;

    this.value.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.name);
      if (graph.node[token]) Node.direct(token, key, this);
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        Node.direct(token, key, this);
      }
    });
  }
};
