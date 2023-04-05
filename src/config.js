const fs = require("fs");
const { argv } = require("yargs");
const home = require("os").homedir();

const root = `${home}/.nuc`;

// `config` source sequence: options, arguments, config.json and defaults
let config = {
  path: {
    root,
    data: `${root}/data`,
    openapi: `${root}/openapi`,
    extensions: `${root}/extensions`,
  },
  port: {
    terminal: 8448,
    cluster: 4000,
  },
};

try {
  const json = require(`${config.path.root}/config.json`);
  config = { ...config, ...json };
} catch (err) {} // eslint-disable-line

if (argv.terminalPort) {
  config.port.terminal = argv.terminalPort;
}

if (argv.clusterPort) {
  config.port.cluster = argv.clusterPort;
}

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
