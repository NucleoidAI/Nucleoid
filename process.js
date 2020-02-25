const fs = require("fs");
const argv = require("yargs").argv;
var nucleoid = require("./nucleoid");

if (fs.existsSync("./data/" + argv.id)) {
  fs.readFileSync("./data/" + argv.id, "utf8")
    .split(/\n/)
    .forEach(line => {
      try {
        let details = JSON.parse(line);
        nucleoid.run(details.s, false, true);
      } catch (error) {
        return;
      }
    });
} else if (fs.existsSync("./data/process")) {
  fs.readFileSync("./data/process", "utf8")
    .split(/\n/)
    .forEach(line => {
      try {
        let details = JSON.parse(line);
        nucleoid.run(details.s);
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
