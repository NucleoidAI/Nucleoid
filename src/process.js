const fs = require("fs");
const { argv } = require("yargs");
const runtime = require("./runtime");
const state = require("./state");
const File = require("./file");
const path = File.data;

let _options = {
  cacheOnly: argv.cacheOnly,
};

setImmediate(() => {
  if (_options.test) return;

  const singleton = !argv.id;
  const id = argv.id || "main";
  const PROCESS_PATH = `${path}/${id}`;

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
          runtime.process(details.s, options);

          if (details.x) {
            details.x.map((exec) => state.run(null, exec));
          }
        } catch (error) {
          return;
        }
      });
  }

  if (!singleton) {
    process.on("message", (message) => {
      const config = { details: true };
      let details = runtime.process(message, config);
      process.send(
        JSON.stringify({
          r: details.result,
          d: details.date,
          t: details.time,
          e: details.error,
          m: details.messages,
          v: details.events,
        })
      );
    });
  }
});

const options = (options) => {
  if (options) {
    _options = options || {};

    if (_options.test) {
      _options.cacheOnly = true;
      _options.terminal = false;
    }
    return _options;
  } else return _options;
};

module.exports = { options };
