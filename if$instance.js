var graph = require("./graph");
var IF = require("./if");
var Node = require("./node");

class IF$INSTANCE extends IF {
  run(scope) {
    scope.instance[this.class.name] = this.instance;
    if (this.condition.run(scope, this.instance)) return this.true;
  }

  graph() {
    let list = [];

    this.condition.tokens.forEach(token => {
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

IF$INSTANCE.prototype.type = "INSTANCE";
module.exports = IF$INSTANCE;
