var graph = require("./graph");
var LET = require("./let");

class LET$INSTANCE extends LET {
  run(scope) {
    let expression = this.value.run(scope, this.instance); // eslint-disable-line no-unused-vars
    eval("scope.local." + this.name + "=expression");
  }

  graph() {
    let list = [];

    this.value.tokens.forEach(token => {
      token = token.replace(/.+?(?=\.)/, this.instance.name);
      if (graph.node[token]) list.push(token);
    });

    return list;
  }
}

LET$INSTANCE.prototype.type = "INSTANCE";
module.exports = LET$INSTANCE;
