const fs = require("fs");
const home = require("os").homedir();

if (!fs.existsSync(`${home}/.nucleoid/data`))
  fs.mkdirSync(`${home}/.nucleoid/data`, { recursive: true });

if (!fs.existsSync(`${home}/.nucleoid/openapi`))
  fs.mkdirSync(`${home}/.nucleoid/openapi`, { recursive: true });

module.exports.config = {};
module.exports.data = `${home}/.nucleoid/data`;
module.exports.openapi = `${home}/.nucleoid/openapi`;
