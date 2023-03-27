const fs = require("fs");
const home = require("os").homedir();

const root = `${home}/.nuc`;

const config = {
  path: {
    root,
    data: `${root}/data`,
    openapi: `${root}/openapi`,
    handlers: `${root}/handlers`,
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

if (!fs.existsSync(config.path.handlers)) {
  fs.mkdirSync(config.path.handlers, { recursive: true });
}

module.exports = config;
