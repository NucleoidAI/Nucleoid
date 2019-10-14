var graph = require("./graph");
var IF = require("./if");
var Node = require("./node");
var EXPRESSION = require("./expression");

class IF$INSTANCE extends IF {
  run(scope) {
    this.condition = new EXPRESSION(
      this.declaration.condition.tokens.map(token => {
        let parts = token.split(".");
        if (parts[0] == this.class.name) parts[0] = this.instance.identifier();
        return parts.join(".");
      })
    );

    scope.instance[this.class.name] = this.instance;
    if (this.condition.run(scope, this.instance)) return this.true;
  }

  graph() {
    return this.condition.graph().filter(token => {
      if (graph.node[token]) return true;
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        return true;
      }
    });
  }
}

IF$INSTANCE.prototype.type = "INSTANCE";
module.exports = IF$INSTANCE;
