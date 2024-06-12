const { argv } = require("yargs");
const datastore = require("./src/datastore");
const nucleoid = require("./");
const cluster = require("./src/cluster");
const chalk = require("chalk");

if (!argv.debug) {
  console.debug = () => {};
}

if (argv.silence) {
  console.log = () => {};
  console.info = () => {};
  console.warn = () => {};
  console.error = () => {};
  console.debug = () => {};
}

if (argv.clear) {
  datastore.clear();
}

nucleoid.start().then(() => {
  const { id, port } = require("./src/config")();
  console.log("");
  console.log("---");
  console.log("Terminal: " + chalk.blue(`http://localhost:${port.terminal}`));
  console.log(
    "IDE:      " +
      chalk.blue(`https://nucleoid.com/ide/${id}/query?mode=terminal`)
  );
  console.log("");

  if (argv.cluster) {
    cluster.init();
  }
});
