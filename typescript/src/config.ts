import fs from "fs";
import { argv as yargsArgv } from "yargs";

type CliArgs = {
  id?: string;
  terminalPort?: number;
  clusterPort?: number;
  [x: string]: unknown;
};

const argv = yargsArgv as CliArgs; // type assertion for expected CLI args
import { v4 as uuid } from "uuid";
import os from "os";
import { deepMerge } from "./lib/deep";

const home: string = os.homedir();

type Config = {
  path: string;
  port: {
    terminal: number;
    cluster: number;
    openapi: number;
  };
  options: Record<string, unknown>;
  cache: boolean;
  data: {
    encryption: boolean;
  };
  id?: string;
};

const defaultConfig: Config = {
  path: `${home}/.nuc`,
  port: {
    terminal: 8448,
    cluster: 4000,
    openapi: 3000,
  },
  options: {},
  cache: false,
  data: {
    encryption: true,
  },
};

let _config: Config = { ...defaultConfig };

function init(config: Partial<Config> & { test?: boolean; id?: string } = {}): Config {
  _config = deepMerge(defaultConfig, config) as Config;

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
    }
  }
  if (typeof id === 'string') {
    fs.writeFileSync(`${_config.path}/default`, id);
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

function options(): Record<string, unknown> {
  return _config.options;
}

const getConfig = (): Config => _config;
export default getConfig;
export { init, options };
