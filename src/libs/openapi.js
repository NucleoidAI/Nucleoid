const express = require("express");
const OpenAPI = require("express-openapi");
const { openapi } = require("../file");
const fs = require("fs");
const swagger = require("swagger-ui-express");
const uuid = require("uuid").v4;
const path = require("path");
let nucleoid;

setImmediate(() => {
  nucleoid = require("../../index");
});

let server;
let started = false;

const start = (nuc) => {
  if (started) {
    stop();
  }

  if (!nuc || !nuc.functions) {
    throw Error("Invalid NUC file");
  }

  const { functions } = nuc;
  Object.values(functions).forEach((fn) => {
    try {
      nucleoid.run(fn.code, { declarative: false });
    } catch (error) {
      console.info("Problem occurred while loading NUC file", error);
    }
  });

  let nucleoidPath = path.dirname(__dirname);

  if (process.platform === "win32") {
    nucleoidPath = nucleoidPath.split("\\");
  } else {
    nucleoidPath = nucleoidPath.split("/");
  }

  nucleoidPath.pop();
  nucleoidPath = nucleoidPath.join("/") + "/index";

  const tmp = uuid();

  const app = express();
  app.use(express.json());

  const { api } = nuc;

  Object.entries(api).forEach(([key, value]) => {
    const parts = key.substring(1).split("/");
    const resource = parts.pop() || "index";
    const path = parts.join("/");

    if (!fs.existsSync(`${openapi}/${tmp}/${path}`))
      fs.mkdirSync(`${openapi}/${tmp}/${path}`, { recursive: true });

    const file = `${openapi}/${tmp}/${path}/${resource}.js`;
    fs.appendFileSync(
      file,
      `const nucleoid = require("${nucleoidPath}"); module.exports = function () {`
    );

    Object.entries(value).forEach(([method, nucDoc]) => {
      const { action } = nucDoc;
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
          `const result = nucleoid.run(${action},scope);` +
          `res.send(result);` +
          `}`
      );
      fs.appendFileSync(file, `${method}.apiDoc = ${JSON.stringify(apiDoc)};`);
    });
    fs.appendFileSync(file, `return { ${Object.keys(value)} };`);
    fs.appendFileSync(file, `}`);
  });

  OpenAPI.initialize({
    app,
    apiDoc: {
      openapi: "3.0.1",
      info: {
        title: "Nucleoid",
        version: "1.0.0",
      },
      components: {
        schemas: nuc.types,
      },
      paths: {},
      servers: [
        {
          url: "/api",
        },
      ],
    },
    paths: `${openapi}/${tmp}`,
    docsPath: "/openapi.json",
  });

  app.use(
    "/",
    swagger.serve,
    swagger.setup(null, {
      swaggerOptions: {
        url: "/api/openapi.json",
      },
    })
  );

  server = app.listen(3000);
  started = true;
};

const stop = () => {
  if (!started) return;

  server.close();
  started = false;
};

const status = () => ({ started });

module.exports.start = start;
module.exports.stop = stop;
module.exports.status = status;
