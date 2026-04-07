export type GroupData = {
  id: string;
  name: string;
};

export class Group {
  public constructor(
    public readonly id: string,
    private _name: string,
  ) {}

  public get name() {
    return this._name;
  }
}
