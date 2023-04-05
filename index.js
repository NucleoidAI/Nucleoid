const nucleoid = require("./src/nucleoid");
const express = require("./src/express");
const config = require("./src/config");

module.exports = express;
module.exports.start = nucleoid.start;
module.exports.register = nucleoid.register;
module.exports.run = nucleoid.run;
module.exports.config = config;
