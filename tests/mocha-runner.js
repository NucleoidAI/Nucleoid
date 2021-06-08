const Mocha = require("mocha");
const mocha = new Mocha();

mocha.addFile(`${process.cwd()}/nucleoid.spec.js`);
mocha.run();
