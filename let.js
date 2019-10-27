var graph = require("./graph");
var Node = require("./node");

class LET {
  prepare() {}
  run(scope) {
    let value = this.value.run(scope); // eslint-disable-line no-unused-vars
    eval("scope.local." + this.name + "=value");
  }

  graph(scope) {
    eval("scope.graph." + this.name + "=this");
    return this.value.graph(scope).filter(token => {
      if (graph[token]) return true;
      else if (graph[token.split(".")[0]]) {
        graph[token] = new Node();
        return true;
      }
    });
  }
}

LET.prototype.type = "REGULAR";
module.exports = LET;
