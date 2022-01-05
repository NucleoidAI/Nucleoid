const fs = require("fs");
const home = require("os").homedir();

if (!fs.existsSync(`${home}/.nucleoid/data`))
  fs.mkdirSync(`${home}/.nucleoid/data`, { recursive: true });

module.exports.config = {};
module.exports.data = `${home}/.nucleoid/data`;
module.exports.openapi = "./openapi";
