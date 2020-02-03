const fs = require("fs");
const argv = require("yargs").argv;
var nucleoid = require("./nucleoid.js");

if (fs.existsSync(argv.path)) {
  fs.readFileSync(argv.path, "utf8")
    .split(/\n/)
    .forEach(line => {
      nucleoid.run(line);
    });
}

process.on("message", message => {
  try {
    var result = nucleoid.run(message);
  } catch (error) {
    if (error instanceof Error) {
      process.send(error.message);
    } else {
      process.send(error);
    }

    return;
  }

  if (result === undefined) {
    process.send("");
  } else {
    process.send(JSON.stringify(result));
  }
});
