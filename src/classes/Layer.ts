import { invoke } from '@tauri-apps/api/core';

export class Layer {
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

  public async setColor(color: string) {
    this._color = color;
    await invoke('set_layer_color', {
      layer: this.id,
      color,
    });
  }
}
