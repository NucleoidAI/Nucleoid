import express from "express";
import cors from "cors";
import OpenAPI from "express-openapi";
import fs from "fs";
import swagger from "swagger-ui-express";
import { v4 as uuid } from "uuid";
import path from "path";
import config from "../config";

let server: ReturnType<typeof express['listen']>;
let started = false;
let _app: ReturnType<typeof express> | null = null;

function init(app?: ReturnType<typeof express>): void {
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

function load(spec: Record<string, any>): void {
  const { "x-nuc-prefix": prefix = "", "x-nuc-events": events = [] } = spec;

  const app = _app;
  const _config = config();

  if (!app) {
    throw Error("OpenAPI has not been initialized");
  }

  const root = path
    .dirname(__dirname)
    .split(process.platform === "win32" ? "\\" : "/")
    .slice(0, -1)
    .join("/");

  const tmp = uuid();

  Object.entries(spec?.paths).forEach(([key, value]) => {
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
      `const nucleoid = require("${root}"); module.exports = function () {`
    );

    Object.entries(value).forEach(([method, nucDoc]) => {
      const action = nucDoc["x-nuc-action"];

      fs.appendFileSync(
        file,
        `function ${method}(req, res) {` +
          `const scope = { params: req.params, query: req.query, body: req.body, user: req.headers["x-nuc-user"] };` +
          `const { result, error } = nucleoid.run(${action}, scope, { details: true });` +
          `if (!result) res.status(${method === "get" ? 404 : 200}).end();` +
          `else if (error) res.status(400).json({ error: result });` +
          `else res.status(200).json(result);` +
          `}`
      );
      fs.appendFileSync(
        file,
        `${method}.apiDoc = ${JSON.stringify(spec.paths[key][method])};`
      );
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
        components: spec.components,
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

  app.use(
    `${prefix}/`,
    swagger.serve,
    swagger.setup(null, {
      swaggerOptions: {
        url: `${prefix}/api/openapi.json`,
      },
      customCssUrl:
        "https://cdn.jsdelivr.net/npm/swagger-ui-themes@3.0.0/themes/3.x/theme-flattop.css",
    })
  );

  try {
    const eventExtension = require(`${_config.path}/extensions/event.js`);
    eventExtension.listen(events);
  } catch (err) {} // eslint-disable-line no-empty
}

function app(): ReturnType<typeof express> | null {
  return _app;
}

function start(port: number = 3000): void {
  if (started) {
    return stop();
  }

  server = _app.listen(port);
  started = true;
}

function stop(): void {
  if (!started) return;

  server.close();
  started = false;
}

function status(): { started: boolean } {
  return { started };
}

export { init, load, app, start, stop, status };
