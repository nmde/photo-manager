export type SettingKey = 'encrypt' | 'theme';

export class Setting {
  public constructor(
    private id: string,
    private _setting: SettingKey,
    private _value: number,
  ) {}

  public get setting() {
    return this._setting;
  }

  public get value() {
    return this._value;
  }
}
