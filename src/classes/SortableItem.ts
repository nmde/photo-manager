import type { Nullable } from '@/types';

export class SortableItem {
  public constructor(
    public id: string,
    public count: number,
    public _name: string,
    public _photo: Nullable<string>,
  ) {}

  public get name() {
    return this._name;
  }

  public get photo() {
    return this._photo;
  }

  public set name(value: string) {
    this._name = value;
  }
}
