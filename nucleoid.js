var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./state").state.graph;
var Node = require("./state").state.Node;
var Statement = require("./statement");

module.exports.run = function(string) {
  let statement = new Statement(string);

  if (statement.next() == "var") {
    statement.skip();
  }

  if (statement.check() == "=") {
    statement.mark();

    if (!graph[statement.token]) {
      graph[statement.token] = new Node();
    }

    let variable = statement.token;

    statement.scan(function(token) {
      if (graph[token]) {
        graph[token].nodes[variable] = graph[variable];

        return true;
      } else {
        return false;
      }
    });

    graph[variable].data = { statement: statement };

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
  } else {
    if (graph[statement.token]) {
      statement.mark();
    }

    statement.scan(token => graph[token]);
    return eval(statement.toString());
  }
};
