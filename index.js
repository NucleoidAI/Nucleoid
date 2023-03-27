const nucleoid = require("./src/nucleoid");
const express = require("./src/express");

module.exports = express;
module.exports.start = nucleoid.start;
module.exports.register = nucleoid.register;
module.exports.run = nucleoid.run;
