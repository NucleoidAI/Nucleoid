import revive from "./lang/$nuc/revive";
import { Data, Config } from "./types";
import cache from "./cache";

const datastore = cache;

function init(config: Config = { cache: true }): void {
  datastore.init(config);
}

function clear(): void {
  datastore.clear();
}

function read(): Data[] {
  return datastore.read().map((data) => ({
    ...data,
    result: {
      $nuc: revive(data.result.$nuc),
      value: data.result.value,
    },
  }));
}

function write(data: Data): Data {
  return datastore.write(data);
}

function tail(n: number): Data[] {
  return datastore.tail(n);
}

export default { init, read, write, clear, tail };
