const fs = require("fs");
const { argv } = require("yargs");

if (argv.clear) {
  const path = `${require("os").homedir()}/.nuc/data`;
  fs.rmSync(path, { recursive: true, force: true });
}

const nucleoid = require("./src/nucleoid");

nucleoid.start();

if (argv.cluster) {
  require("./src/cluster");
}
