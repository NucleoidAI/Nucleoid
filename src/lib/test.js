const datastore = require("@nucleoidjs/datastore");
const { state } = require("../state");
const graph = require("../graph");

const clear = () => {
  for (let property in state) delete state[property];
  for (let property in graph) delete graph[property];

  state["classes"] = [];
  graph["classes"] = { name: "classes" };

  datastore.clear();
};

clear();

module.exports = { clear };
