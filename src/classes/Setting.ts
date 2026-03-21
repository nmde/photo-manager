export type SettingKey = 'theme' | 'version';

export type SettingData = {
  id: string;
  setting: SettingKey;
  value: number;
};
