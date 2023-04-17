const express = require("express");
const cors = require("cors");
const OpenAPI = require("express-openapi");
const fs = require("fs");
const swagger = require("swagger-ui-express");
const uuid = require("uuid").v4;
const path = require("path");
const config = require("../config");

let server;
let started = false;
let _app = null;

function init(app) {
  if (app) {
    _app = app;
  } else {
    if (started) {
      stop();
    }

    _app = express();
    _app.use(express.json());
    _app.use(cors());
  }
}

function load({ api, types, prefix = "" }) {
  const app = _app;
  const _config = config();

  if (!app) {
    throw Error("OpenAPI has not been initialized");
  }

  const root = path
    .dirname(__dirname)
    .split(process.platform === "win32" ? "\\" : "/")
    .slice(0, -1);

  const nucleoidPath = root.join("/") + "/index";

  const tmp = uuid();

  Object.entries(api).forEach(([key, value]) => {
    const parts = key.substring(1).split("/");
    const resource = parts.pop() || "index";
    const path = parts.join("/");

    if (!fs.existsSync(`${_config.path}/openapi/${tmp}/${path}`))
      fs.mkdirSync(`${_config.path}/openapi/${tmp}/${path}`, {
        recursive: true,
      });

    const file = `${_config.path}/openapi/${tmp}/${path}/${resource}.js`;
    fs.appendFileSync(
      file,
      `const nucleoid = require("${nucleoidPath}"); module.exports = function () {`
    );

    Object.entries(value).forEach(([method, nucDoc]) => {
      const action = nucDoc["x-nuc-action"];
      const params = nucDoc.params || [];

      const apiDoc = {
        ...nucDoc,
        requestBody:
          method !== "get"
            ? {
                content: {
                  "application/json": {
                    schema: nucDoc.request,
                  },
                },
              }
            : undefined,
        responses: {
          200: {
            description: "Successful Operation",
            content: {
              "application/json": {
                schema: nucDoc.response,
              },
            },
          },
        },
        parameters: params.map((param) => ({
          ...param,
          schema: { type: param.type },
          type: undefined,
        })),
        request: undefined,
        response: undefined,
        action: undefined,
        params: undefined,
      };

      fs.appendFileSync(
        file,
        `function ${method}(req, res) {` +
          `const scope = { params: req.params, query: req.query, body: req.body };` +
          `const { result, error } = nucleoid.run(${action}, scope, { details: true });` +
          `if (!result) res.status(404).end();` +
          `else if (error) res.status(400).json({ error: result });` +
          `else res.status(200).json(result);` +
          `}`
      );
      fs.appendFileSync(file, `${method}.apiDoc = ${JSON.stringify(apiDoc)};`);
    });
    fs.appendFileSync(file, `return { ${Object.keys(value)} };`);
    fs.appendFileSync(file, `}`);
  });

  try {
    OpenAPI.initialize({
      app,
      apiDoc: {
        openapi: "3.0.1",
        info: {
          title: "Nucleoid",
          version: "1.0.0",
        },
        components: {
          schemas: types,
        },
        paths: {},
        servers: [
          {
            url: `${prefix}/api`,
          },
        ],
      },
      paths: `${_config.path}/openapi/${tmp}`,
      docsPath: "/openapi.json",
    });
  } catch (err) {
    console.error(err);
    throw err;
  }

  const { native } = _config;

  if (native) {
    const nucleoid = require("../../");
    fs.writeFileSync(
      `${_config.path}/native/index.js`,
      "let _nucleoid;" +
        "module.exports = (nucleoid) => (_nucleoid = nucleoid);" +
        "module.exports.nucleoid = _nucleoid;"
    );
    require(`${_config.path}/native/`)(nucleoid);

    for (const route of native.routes) {
      const native = require(`${_config.path}/native/${route}`);
      app.use(prefix, native);
    }
  }

  app.use(
    `${prefix}/`,
    swagger.serve,
    swagger.setup(null, {
      swaggerOptions: {
        url: `${prefix}/api/openapi.json`,
      },
    })
  );
}

function app() {
  return _app;
}

function start(port = 3000) {
  if (started) {
    return stop();
  }

  server = _app.listen(port);
  started = true;
}

function stop() {
  if (!started) return;

  server.close();
  started = false;
}

function status() {
  return { started };
}

module.exports.init = init;
module.exports.load = load;
module.exports.app = app;
module.exports.start = start;
module.exports.stop = stop;
module.exports.status = status;
