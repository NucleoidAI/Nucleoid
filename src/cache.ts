import { Config, Data } from "./types";

let cache: string[] = [];

function init(config: Config) {
  if (config.cache) {
    console.log("Cache is enabled");
  }

  return;
}

function read(): Data[] {
  return cache.map((data) => JSON.parse(data));
}

function write(data: Data): Data {
  cache.push(JSON.stringify(data));
  return data;
}

function clear() {
  cache = [];
}

function tail(n = 10): Data[] {
  return cache
    .map((data) => JSON.parse(data))
    .reverse()
    .slice(0, n);
}

export default { init, read, write, clear, tail };
