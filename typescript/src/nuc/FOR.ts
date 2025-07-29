import _ from "lodash";
import * as graph from "../graph";
import * as state from "../state";
import { Evaluation } from "../lang/Evaluation";
import { Instruction } from "../Instruction";
import $BLOCK from "../lang/$nuc/$BLOCK";
import $LET from "../lang/$nuc/$LET";

interface Scope {
  [key: string]: string | number | boolean | object | null;
}

class FOR {
  private index: number;
  public array: string;
  public variable: { node: object };
  public statements: object[];

  constructor() {
    this.index = 0;
  }

  run(scope: Scope): { next: object[] } | null {
    const array: string[] | number[] | boolean[] | object[] = state.expression(
      scope,
      new Evaluation(`state.${this.array}`)
    );

    if (!Array.isArray(array)) {
      throw new TypeError(`${this.array} is not iterable`);
    }

    if (this.index < array.length) {
      const list: object[] = [];
      const key: string | number | null = array[this.index].id;

      if (key !== undefined && graph.$[key]) {
        const object: string | number | null = array[this.index++].id;
        const statements: object[] = [$LET(this.variable.node, object)];
        list.push(
          $BLOCK(statements.concat(_.cloneDeep(this.statements)), true)
        );
      } else {
        this.index++;
      }

      list.push(new Instruction(scope, this, false, true, false, false));
      return { next: list };
    }

    return null;
  }
}

export default FOR;
