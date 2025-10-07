export class Activity {
  public constructor(private _id: string, private _icon: string, private _name: string) {}

  public get id() {
    return this._id;
  }

  public get icon() {
    return this._icon;
  }

  public get name() {
    return this._name;
  }
}
