var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var PROPERTY = require("./property");

class PROPERTY$INSTANCE extends PROPERTY {
  run(scope) {
    this.id = this.instance.name + "." + this.name;

    let instance = state[this.instance.name];
    instance[this.name] = this.value.run(scope, this.instance);
  }

  graph() {
    let list = [];

    this.value.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.name);
      if (graph.node[token]) list.push(token);
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        list.push(token);
      }
    });

    return list;
  }
}

PROPERTY$INSTANCE.prototype.type = "INSTANCE";
module.exports = PROPERTY$INSTANCE;
