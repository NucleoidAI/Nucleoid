import { Config, Options } from "./types";

declare function init(config: Config): Config;
declare function get(): Config;
declare function options(): Options;

export default get;
export { init, options };
