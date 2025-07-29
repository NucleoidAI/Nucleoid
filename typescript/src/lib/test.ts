import datastore from "../datastore";
import state from "../state";
import graph from "../graph";

const clear = () => {
  state.clear();
  graph.clear();
  datastore.clear();
};

export default { clear };
