const datastore = require("../datastore");
const state = require("../state");
const graph = require("../graph");

const clear = () => {
  state.clear();
  graph.clear();
  datastore.clear();
};

module.exports = { clear };
