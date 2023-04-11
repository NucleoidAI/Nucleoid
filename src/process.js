const datastore = require("@nucleoidjs/datastore");
const runtime = require("./runtime");
const state = require("./state");
const config = require("./config");

function init() {
  const { id } = config();
  datastore.init({ id });

  setImmediate(() => {
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
  });
}

module.exports.init = init;
