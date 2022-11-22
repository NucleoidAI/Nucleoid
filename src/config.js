const fs = require("fs");
const home = require("os").homedir();

if (!fs.existsSync(`${home}/.nuc/openapi`)) {
  fs.mkdirSync(`${home}/.nuc/openapi`, { recursive: true });
}

const config = {
  path: {
    data: `${home}/.nuc/data`,
    openapi: `${home}/.nuc/openapi`,
  },
};

module.exports = config;
