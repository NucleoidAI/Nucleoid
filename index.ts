import nucleoid from "./src/nucleoid";
import express from "./src/express";
import config from "./src/config";
import test from "./src/lib/test";
import openapi from "./src/lib/openapi";
import datastore from "./src/datastore";

const { start, run } = nucleoid;
export default {
  express,
  start,
  run,
  config,
  openapi,
  test,
  datastore,
};
