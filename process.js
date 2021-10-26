const fs = require("fs");
const argv = require("yargs").argv;
const nucleoid = require("./nucleoid");

if (fs.existsSync(`${argv.path}/${argv.id}`)) {
  fs.readFileSync(`${argv.path}/${argv.id}`, "utf8")
    .split(/\n/)
    .forEach((line) => {
      try {
        let details = JSON.parse(line);
        nucleoid.run(details.s, false, true);
      } catch (error) {
        return;
      }
    });
}

process.on("message", (message) => {
  let details = nucleoid.run(message, true);
  process.send(
    JSON.stringify({
      r: details.result,
      t: details.time,
      m: details.messages,
      v: details.events,
      e: details.error,
    })
  );
});
