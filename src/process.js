const datastore = require("@nucleoidjs/datastore");
const runtime = require("./runtime");
const state = require("./state");
const config = require("./config");
const openapi = require("./lib/openapi");
const context = require("./lib/context");
const terminal = require("./terminal");

function init() {
  const _config = config();
  datastore.init(_config);

  setImmediate(() => {
    datastore.read().forEach((details) => {
      const options = {
        declarative: !!details.c,
        cacheOnly: true,
      };

      if (details.w && !details.e) {
        runtime.process(details.s, options);

        if (details.j) {
          details.j.map((adjust) => state.run(null, adjust));
        }
      }
    });

    const nucleoid = require("../");
    require(`${_config.path}/nucleoid.js`)(nucleoid);

    try {
      const event = require(`${_config.path}/extensions/event.js`);
      event.init();
    } catch (err) {} // eslint-disable-line no-empty

    const { native, port, test } = _config;

    if (native) {
      try {
        for (const route of native.routes) {
          const native = require(`${_config.path}/native/${route}`);
          terminal.use(native);
        }
      } catch (err) {
        console.error("Failed to load native routes");
        throw err;
      }
    }

    try {
      const {
        api,
        types,
        prefix,
        port,
        functions,
        events,
      } = require(`${_config.path}/openapi.json`);

      if (functions) {
        context.run(functions);
      }

      openapi.init();
      openapi.load({ api, types, prefix, events });
      openapi.start(port || _config.port.openapi);
    } catch (err) {
      if (err.code !== "MODULE_NOT_FOUND") {
        console.error("Error loading OpenAPI", err);
        process.exit(1);
      }
    }

    if (!test) {
      terminal.all("*", (req, res) => res.status(404).end());
      terminal.listen(port.terminal);
    }
  });
}

module.exports.init = init;
