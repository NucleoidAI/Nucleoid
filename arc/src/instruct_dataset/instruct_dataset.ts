import core from "./dataset.core.json";
import line from "./dataset.line.json";
import curve from "./dataset.curve.json";

export function document(): string {
  return `
    instruct_dataset:
    ${JSON.stringify(core)}      
    ${JSON.stringify(line)}      
    ${JSON.stringify(curve)}      
  `;
}
