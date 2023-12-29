const fs = require("fs");
const { argv } = require("yargs");
const { v4: uuid } = require("uuid");
const home = require("os").homedir();
const { deepMerge } = require("./lib/deep");

let defaultConfig = {
  path: `${home}/.nuc`,
  port: {
    terminal: 8448,
    cluster: 4000,
    openapi: 3000,
  },
  declarative: false,
  details: false,
  cache: false,
  data: {
    encryption: true,
  },
};

let _config = { ...defaultConfig };

function init(config = {}) {
  _config = deepMerge(defaultConfig, config);

  if (!fs.existsSync(_config.path)) {
    fs.mkdirSync(_config.path, { recursive: true });
  }

  require("dotenv").config({ path: `${_config.path}/.env` });

  if (!fs.existsSync(`${_config.path}/data`)) {
    fs.mkdirSync(`${_config.path}/data`, { recursive: true });
  }

  if (!fs.existsSync(`${_config.path}/openapi`)) {
    fs.mkdirSync(`${_config.path}/openapi`, { recursive: true });
  }

  if (!fs.existsSync(`${_config.path}/native`)) {
    fs.mkdirSync(`${_config.path}/native`, { recursive: true });
  }

  if (!fs.existsSync(`${_config.path}/extensions`)) {
    fs.mkdirSync(`${_config.path}/extensions`, { recursive: true });
  }

  fs.writeFileSync(
    `${_config.path}/nucleoid.js`,
    "/* eslint-disable */ let _nucleoid; module.exports = (nucleoid) => { if (nucleoid) { _nucleoid = nucleoid; } return _nucleoid; };"
  );

  if (config.test) {
    _config.id = config.id || uuid();
    _config.cache = true;
  } else {
    try {
      const json = require(`${_config.path}/config.json`);
      _config = { ..._config, ...json };
    } catch (err) {} // eslint-disable-line no-empty
  }

  let id = argv.id || _config.id;

  if (!id) {
    try {
      id = fs.readFileSync(`${_config.path}/default`, "utf8").trim();
    } catch (err) {
      id = uuid();
      fs.writeFileSync(`${_config.path}/default`, id);
    }
  }

  _config.id = id;

  if (argv.terminalPort) {
    _config.port.terminal = argv.terminalPort;
  }

  if (argv.clusterPort) {
    _config.port.cluster = argv.clusterPort;
  }

  return _config;
}

module.exports = () => _config;
module.exports.init = init;
