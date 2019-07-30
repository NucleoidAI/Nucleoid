var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var ASSIGNMENT = require("./assignment");

module.exports = class ASSIGNMENT$INSTANCE extends ASSIGNMENT {
  run(scope) {
    let instance = state[this.instance.variable];
    instance[this.property] = this.expression.run(scope.local, this.instance);
  }

  graph() {
    let key = this.instance.variable + "." + this.property;
    graph.node[key] = this;

    this.expression.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.variable);
      if (graph.node[token]) Node.direct(token, key, this);
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        Node.direct(token, key, this);
      }
    });
  }
};
