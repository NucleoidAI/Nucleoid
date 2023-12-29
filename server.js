const { argv } = require("yargs");
const datastore = require("./src/datastore");
const nucleoid = require("./");
const cluster = require("./src/cluster");

if (!argv.debug) {
  console.debug = () => {};
}

if (argv.clear) {
  datastore.clear();
}

nucleoid.start().then(() => {
  console.log("Nucleoid runtime is started");

  if (argv.cluster) {
    cluster.init();
  }
});
