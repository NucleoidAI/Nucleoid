const fs = require("fs");
const { argv } = require("yargs");

if (argv.clear) {
  const path = `${require("os").homedir()}/.nuc/data`;
  fs.rmSync(path, { recursive: true, force: true });
  fs.mkdirSync(path, { recursive: true });
}

const nucleoid = require("./index");

nucleoid.start();

if (argv.cluster) {
  const process = require("./src/cluster/process");
  nucleoid.register(process);
}
