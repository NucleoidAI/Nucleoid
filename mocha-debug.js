var Mocha = require("mocha");
var mocha = new Mocha();

mocha.addFile("/home/ubuntu/environment/nucleoid/tests/nucleoid.spec.js");
mocha.run();
