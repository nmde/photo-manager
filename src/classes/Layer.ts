import { invoke } from '@tauri-apps/api/core';

export type LayerData = {
  id: string;
  name: string;
  color: string;
};

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

  public static createLayers(data: LayerData[]) {
    const layers: Record<string, Layer> = {};
    for (const layer of data.map(({ id, name, color }) => new Layer(id, name, color))) {
      layers[layer.id] = layer;
    }
    return layers;
  }

  public async setColor(color: string) {
    this._color = color;
    await invoke('set_layer_color', {
      layer: this.id,
      color,
    });
  }
}
