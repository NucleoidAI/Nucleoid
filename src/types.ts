import $ from "./lang/$nuc/$";

type Options = {
  declarative?: boolean;
  details?: boolean;
};

type Config = {
  path?: string;
  port?: {
    terminal?: number;
    cluster?: number;
    openapi?: number;
  };
  options?: Options;
  cache?: boolean;
  data?: {
    encryption?: boolean;
  };
  id?: string;
  test?: boolean;
};

type Event = {
  topic: string;
  data: string;
};

type Result = {
  $nuc: $[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value: any;
};

type Data = {
  string: string;
  declarative?: boolean;
  result: Result;
  time: number;
  date: Date;
  error?: boolean;
  events: Event[];
};

export { Options, Config, Data, Event };
