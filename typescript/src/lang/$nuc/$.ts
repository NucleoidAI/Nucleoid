// TODO Rename to $NODE
import NODE from "../../nuc/NODE";

class $ {
  type: string;
  public iof: string;
  private pre: boolean;
  asg: boolean = false;

  constructor() {
    this.type = this.constructor.name;
    this.iof = this.constructor.name;
    this.pre = false;
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  before(scope): void {}
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  run(scope): NODE | NODE[] | $ | null {
    return null;
  }
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  graph(scope): void {}
  after(): void {}

  get prepared(): boolean {
    return this.pre;
  }

  set prepared(prepared: boolean) {
    this.pre = prepared;
  }
}

export default $;
