const fs = require("fs");
const argv = require("yargs").argv;
var nucleoid = require("./nucleoid.js");

if (fs.existsSync("./data/" + argv.id)) {
  fs.readFileSync("./data/" + argv.id, "utf8")
    .split(/\n/)
    .forEach(line => {
      nucleoid.run(line);
    });
} else if (fs.existsSync("./data/process")) {
  fs.readFileSync("./data/process", "utf8")
    .split(/\n/)
    .forEach(line => {
      try {
        nucleoid.run(line);
      } catch (error) {
        return;
      }
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
