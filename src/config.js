const fs = require("fs");
const { argv } = require("yargs");
const home = require("os").homedir();

const root = `${home}/.nuc`;

const config = {
  path: {
    root,
    data: `${root}/data`,
    openapi: `${root}/openapi`,
    extensions: `${root}/extensions`,
  },
  port: {
    terminal: argv.port || 8448,
    cluster: 4000,
  },
};

if (!fs.existsSync(config.path.root)) {
  fs.mkdirSync(config.path.root, { recursive: true });
}

if (!fs.existsSync(config.path.data)) {
  fs.mkdirSync(config.path.data, { recursive: true });
}

if (!fs.existsSync(config.path.openapi)) {
  fs.mkdirSync(config.path.openapi, { recursive: true });
}

if (!fs.existsSync(config.path.extensions)) {
  fs.mkdirSync(config.path.extensions, { recursive: true });
}

module.exports = config;
