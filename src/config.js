const fs = require("fs");
const home = require("os").homedir();

const config = {
  path: {
    data: `${home}/.nuc/data`,
    openapi: `${home}/.nuc/openapi`,
    handlers: `${home}/.nuc/handlers`,
  },
};

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
