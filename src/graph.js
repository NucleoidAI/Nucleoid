const graph = {
  classes: {
    name: "classes",
  },
};

function retrieve(variable) {
  if (typeof variable === "string") {
    return graph[variable];
  } else {
    return graph[variable.generate()];
  }
}

function clear() {
  for (let property in graph) {
    delete graph[property];
  }

  graph["classes"] = { name: "classes" };
}

module.exports.graph = graph;
module.exports.retrieve = retrieve;
module.exports.clear = clear;
