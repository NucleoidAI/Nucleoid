const datastore = require("@nucleoidjs/datastore");
const { argv } = require("yargs");
const runtime = require("./runtime");
const state = require("./state");
const config = require("./config");

const id = argv.id || "main";

let _options = {
  id,
  declarative: false,
  details: false,
  cacheOnly: argv.cacheOnly || false,
  port: argv.port || 8448,
};

datastore.init({ id, path: config.path.data });

setImmediate(() => {
  if (_options.test) return;

  const singleton = !argv.id;

  datastore.read().forEach((details) => {
    const options = {
      declarative: !!details.c,
      cacheOnly: true,
    };

    if (!details.e) {
      runtime.process(details.s, options);
    }

    if (details.x) {
      details.x.map((exec) => state.run(null, exec));
    }
  });

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
          h: details.hash,
        })
      );
    });
  }
});

function options(options) {
  if (options) {
    _options = { ..._options, ...options };

    if (_options.test) {
      _options.cacheOnly = true;
      _options.terminal = false;
    }

    return _options;
  } else return _options;
}

module.exports.options = options;
