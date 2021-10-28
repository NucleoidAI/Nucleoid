const express = require("express");
const initialize = require("express-openapi").initialize;
const { openapi } = require("./file");
const fs = require("fs");
const swagger = require("swagger-ui-express");

let server;
let started = false;

const start = (nuc) => {
  if (started) return;

  const app = express();
  app.use(express.json());

  const { api } = nuc;
  fs.rmdirSync(openapi, { recursive: true });

  Object.entries(api).forEach(([key, value]) => {
    const parts = key.substring(1).split("/");
    const resource = parts.pop() || "index";
    const path = parts.join("/");

    if (!fs.existsSync(`${openapi}/${path}`))
      fs.mkdirSync(`${openapi}/${path}`, { recursive: true });

    const file = `${openapi}/${path}/${resource}.js`;
    fs.appendFileSync(
      file,
      `const Service = require("${"../".repeat(
        parts.length + 1
      )}service"); module.exports = function () {`
    );

    Object.entries(value).forEach(([method, nucDoc]) => {
      const { action } = nucDoc;
      const params = nucDoc.params || [];

      const apiDoc = {
        ...nucDoc,
        requestBody: {
          content: {
            "application/json": {
              schema: nucDoc.request,
            },
          },
        },
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
        `function ${method}(req, res){ Service.accept("let json=" + JSON.stringify(req.body) + ";${action}", req, res)}`
      );
      fs.appendFileSync(file, `${method}.apiDoc = ${JSON.stringify(apiDoc)};`);
    });
    fs.appendFileSync(file, `return { ${Object.keys(value)} };`);
    fs.appendFileSync(file, `}`);
  });

  initialize({
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
    paths: `${openapi}`,
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

module.exports.start = start;
module.exports.stop = stop;
