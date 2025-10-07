export class Group {
  public constructor(private id: string, private _name: string) {}

  public get name() {
    return this._name;
  }
}
