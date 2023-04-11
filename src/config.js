const fs = require("fs");
const { argv } = require("yargs");
const { v4: uuid } = require("uuid");
const home = require("os").homedir();

let defaultConfig = {
  path: `${home}/.nuc`,
  port: {
    terminal: 8448,
    cluster: 4000,
  },
  declarative: false,
  details: false,
  cacheOnly: false,
};

let _config = { ...defaultConfig };

function init(config = {}) {
  _config = { ...defaultConfig };
  _config = { ..._config, ...config };

  if (!fs.existsSync(_config.path)) {
    fs.mkdirSync(_config.path, { recursive: true });
  }
  if (!fs.existsSync(`${_config.path}/data`)) {
    fs.mkdirSync(`${_config.path}/data`, { recursive: true });
  }

  if (!fs.existsSync(`${_config.path}/openapi`)) {
    fs.mkdirSync(`${_config.path}/openapi`, { recursive: true });
  }

  if (!fs.existsSync(`${_config.path}/extensions`)) {
    fs.mkdirSync(`${_config.path}/extensions`, { recursive: true });
  }

  if (config.test) {
    _config.id = config.id || uuid();
    _config.cacheOnly = true;
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

    _config.id = id;
  }

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
