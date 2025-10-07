export class PersonCategory {
  public constructor(private _id: string, private _name: string, private _color: string) {}

  public get id() {
    return this._id;
  }

  public get name() {
    return this._name;
  }

  public get color() {
    return this._color;
  }
}
