import { Entity } from './Entity';

export type SettingKey = 'encrypt' | 'theme';

type SettingData = {
  setting: SettingKey;
  value: boolean | string;
};

export class Setting extends Entity<SettingData> {
  public constructor(data: SettingData) {
    super('Setting', data);
    this.primaryKey = 'setting';
  }
}
