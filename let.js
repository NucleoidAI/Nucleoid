var graph = require("./graph");

class LET {
  run(scope) {
    scope.local[this.name] = this.value.run(scope);
  }

  graph() {
    return this.value.tokens.filter(token => graph.node[token]);
  }
}

LET.prototype.type = "REGULAR";
module.exports = LET;
