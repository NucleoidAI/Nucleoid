const { state } = require("../state");
const graph = require("../graph");
const data = require("../libs/data");

const clear = () => {
  for (let property in state) delete state[property];
  for (let property in graph) delete graph[property];

  state["classes"] = [];
  graph["classes"] = { name: "classes" };

  data.clear();
};

clear();

module.exports = { clear };
