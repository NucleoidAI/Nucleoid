const express = require("express");
const OpenAPI = require("express-openapi");
const config = require("../config");
const fs = require("fs");
const swagger = require("swagger-ui-express");
const uuid = require("uuid").v4;
const path = require("path");
const { argv } = require("yargs");

let server;
let started = false;
let _app = null;

const initialize = (app) => {
  if (app) {
    _app = app;
  } else {
    if (started) {
      stop();
    }

    _app = express();
    _app.use(express.json());
  }
};

const load = ({ api, types, prefix = "" }) => {
  const app = _app;

  if (!app) {
    throw Error("OpenAPI has not been initialized");
  }

  const root = path
    .dirname(__dirname)
    .split(process.platform === "win32" ? "\\" : "/")
    .slice(0, -1);

  const nucleoidPath = root.join("/") + "/index";
  const clusterPath = root.join("/") + "/src/cluster/cluster";

  const tmp = uuid();

  if (argv.cluster) {
    Object.entries(api).forEach(([key, value]) => {
      const parts = key.substring(1).split("/");
      const resource = parts.pop() || "index";
      const path = parts.join("/");

      if (!fs.existsSync(`${config.path.openapi}/${tmp}/${path}`))
        fs.mkdirSync(`${config.path.openapi}/${tmp}/${path}`, {
          recursive: true,
        });

      const file = `${config.path.openapi}/${tmp}/${path}/${resource}.js`;
      fs.appendFileSync(
        file,
        `const cluster = require("${clusterPath}"); module.exports = function () {`
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
          `function ${method}(req, res) { cluster(req, res, ${action}) }`
        );
        fs.appendFileSync(
          file,
          `${method}.apiDoc = ${JSON.stringify(apiDoc)};`
        );
      });
      fs.appendFileSync(file, `return { ${Object.keys(value)} };`);
      fs.appendFileSync(file, `}`);
    });
  } else {
    Object.entries(api).forEach(([key, value]) => {
      const parts = key.substring(1).split("/");
      const resource = parts.pop() || "index";
      const path = parts.join("/");

      if (!fs.existsSync(`${config.path.openapi}/${tmp}/${path}`))
        fs.mkdirSync(`${config.path.openapi}/${tmp}/${path}`, {
          recursive: true,
        });

      const file = `${config.path.openapi}/${tmp}/${path}/${resource}.js`;
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
            `else if (error) res.status(400).json(result);` +
            `else res.status(200).json(result);` +
            `}`
        );
        fs.appendFileSync(
          file,
          `${method}.apiDoc = ${JSON.stringify(apiDoc)};`
        );
      });
      fs.appendFileSync(file, `return { ${Object.keys(value)} };`);
      fs.appendFileSync(file, `}`);
    });
  }

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
      paths: `${config.path.openapi}/${tmp}`,
      docsPath: "/openapi.json",
    });
  } catch (err) {
    console.error(err);
    throw err;
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
};

const app = () => _app;

const start = (port = 3000) => {
  if (started) {
    return stop();
  }

  server = _app.listen(port);
  started = true;
};

const stop = () => {
  if (!started) return;

  server.close();
  started = false;
};

const status = () => ({ started });

module.exports = { initialize, load, app, start, stop, status };
