const graph = {
  classes: {
    name: "classes",
  },
};

function retrieve(identifier) {
  if (typeof identifier === "string") {
    return graph[identifier];
  } else {
    return graph[identifier.generate()];
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
