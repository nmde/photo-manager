export type GroupData = {
  id: string;
  name: string;
};

export class Group implements GroupData {
  public constructor(
    public readonly id: GroupData['id'],
    private _name: GroupData['name'],
  ) {}

  public get name() {
    return this._name;
  }
}
