const fs = require("fs");
const argv = require("yargs").argv;
const nucleoid = require("./nucleoid");
const state = require("./state");

const processPath = `${argv.path}/${argv.id}`;
if (fs.existsSync(processPath)) {
  fs.readFileSync(processPath, "utf8")
    .split(/\n/)
    .forEach((line) => {
      try {
        let details = JSON.parse(line);
        const options = {
          declarative: details.c,
          graphOnly: true,
          cacheOnly: true,
        };
        nucleoid.run(details.s, options);

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
