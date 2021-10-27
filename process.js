const fs = require("fs");
const argv = require("yargs").argv;
const nucleoid = require("./nucleoid");
const state = require("./state");

if (fs.existsSync(`${argv.path}/${argv.id}`)) {
  fs.readFileSync(`${argv.path}/${argv.id}`, "utf8")
    .split(/\n/)
    .forEach((line) => {
      try {
        let details = JSON.parse(line);
        const config = {
          declarative: details.c,
          graphOnly: true,
          cacheOnly: true,
        };
        nucleoid.run(details.s, config);

        if (details.x) {
          details.x.map((exec) => state.run(null, exec));
        }
      } catch (error) {
        return;
      }
    });
}

process.on("message", (message) => {
  const config = { details: true, declarative: false };
  let details = nucleoid.run(message, config);
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
