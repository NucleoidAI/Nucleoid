var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./state").state.graph;
var Node = require("./state").state.Node;
var Statement = require("./statement");

module.exports = class Variable extends Statement {
  constructor(statement) {
    super(statement.string);
    this.type = "VAR";
    this.tokens = statement.tokens;
    this.token = statement.token;
    this.offset = statement.offset;
  }

  run() {
    if (!graph[this.token]) {
      graph[this.token] = new Node();
    }

    let variable = this.token;

    this.scan(function(token) {
      if (graph[token]) {
        graph[token].nodes[variable] = graph[variable];

        return true;
      } else {
        return false;
      }
    });

    graph[variable].data = { statement: this };

    let result = eval(graph[variable].data.statement.toString());

    let queue = [];
    queue.push(variable);

    while (queue.length != 0) {
      let element = queue.shift();

      for (let node in graph[element].nodes) {
        queue.push(node);
        eval(graph[node].data.statement.toString());
      }
    }

    return result;
  }
};
