const { argv } = require("yargs");
const datastore = require("@nucleoidjs/datastore");
const nucleoid = require("./");
const cluster = require("./src/cluster");

if (argv.clear) {
  datastore.clear();
}

nucleoid.start();

if (argv.cluster) {
  cluster.init();
}
