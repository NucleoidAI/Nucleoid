const fs = require("fs");
const argv = require("yargs").argv;
var nucleoid = require("./nucleoid");

if (fs.existsSync("/var/lib/nucleoid/" + argv.id)) {
  fs.readFileSync("/var/lib/nucleoid/" + argv.id, "utf8")
    .split(/\n/)
    .forEach(line => {
      try {
        let details = JSON.parse(line);
        nucleoid.run(details.s, false, true);
      } catch (error) {
        return;
      }
    });
} else if (fs.existsSync("/var/lib/nucleoid/process")) {
  fs.readFileSync("/var/lib/nucleoid/process", "utf8")
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
  var details = nucleoid.run(message, true);
  process.send(
    JSON.stringify({ r: details.result, m: details.messages, e: details.error })
  );
});
