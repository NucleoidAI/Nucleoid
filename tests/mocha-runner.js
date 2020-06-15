var Mocha = require("mocha");
var mocha = new Mocha();

mocha.addFile(`${process.cwd()}/nucleoid.spec.js`);
mocha.run();
