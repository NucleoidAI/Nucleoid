const fs = require("fs");
const argv = require("yargs").argv;
var nucleoid = require("./nucleoid");

init: if (fs.existsSync(`${argv.path}/${argv.id}`)) {
  fs.readFileSync(`${argv.path}/${argv.id}`, "utf8")
    .split(/\n/)
    .forEach(line => {
      try {
        let details = JSON.parse(line);
        nucleoid.run(details.s, false, true);
      } catch (error) {
        return;
      }
    });
} else {
  let parts = argv.id.toString().split("/");

  if (parts.length === 1) {
    break init;
  }

  let group = parts[0];

  if (fs.existsSync(`${argv.path}/init/${group}`)) {
    fs.readFileSync(`${argv.path}/init/${group}`, "utf8")
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
}

process.on("message", message => {
  var details = nucleoid.run(message, true);
  process.send(
    JSON.stringify({
      r: details.result,
      t: details.time,
      m: details.messages,
      v: details.events,
      e: details.error
    })
  );
});
