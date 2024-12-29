import { Data, Options } from "./types";
import config from "./config";
import event from "./event";
import datastore from "./datastore";
import Statement from "./lib/statement";
import transaction from "./transaction";
import stack from "./stack";

const defaultOptions: Options = {
  declarative: false,
  details: false,
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function process(string: string, options: Options = {}): Data | any {
  options = { ...defaultOptions, ...config().options, ...options };

  const before = Date.now();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let result: any;
  let error = false;

  try {
    const statements = Statement.compile(string);

    if (!statements.length) {
      return;
    }

    transaction.start();
    result = stack.process(statements, null, options);
    transaction.end();
  } catch (err) {
    transaction.rollback();
    error = true;
    result = err;
  }

  const events = event.list();
  const date = Date.now();
  const time = date - before;

  const data: Data = {
    string,
    declarative: options.declarative,
    result,
    time,
    date: new Date(date),
    error,
    events,
  };

  datastore.write(data);
  event.clear();

  if (options.details) {
    return data;
  } else {
    if (error) {
      throw result;
    }

    return result.value;
  }
}

export default { process };
