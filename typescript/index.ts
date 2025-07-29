import * as openapi from "./src/lib/openapi";

import config from "./src/config";
import datastore from "./src/datastore";
import express from "./src/express";
import nucleoid from "./src/nucleoid";
import test from "./src/lib/test";

const { start, run, register } = nucleoid;

export default {
  nucleoid,
  register,
  express,
  start,
  run,
  config,
  openapi,
  test,
  datastore,
};
