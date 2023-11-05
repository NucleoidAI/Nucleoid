const datastore = require("@nucleoidjs/datastore");
const state = require("../state");
const graph = require("../graph");

const clear = () => {
  state.clear();
  graph.clear();
  datastore.clear();
};

clear();

module.exports = { clear };
