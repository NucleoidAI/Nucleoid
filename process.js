const fs = require("fs");
const argv = require("yargs").argv;
const nucleoid = require("./nucleoid");
const state = require("./state");

const PROCESS_PATH = `${argv.path}/${argv.id}`;

if (fs.existsSync(PROCESS_PATH)) {
  fs.readFileSync(PROCESS_PATH, "utf8")
    .split(/\n/)
    .forEach((line) => {
      try {
        let details = JSON.parse(line);
        const options = {
          declarative: !!details.c,
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
