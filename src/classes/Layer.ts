import { set_layer_color, set_layer_name } from '@/api/places';

export type LayerData = {
  id: string;
  name: string;
  color: string;
  count: number;
};

export class Layer {
  public constructor(
    public readonly id: string,
    public _name: string,
    public _color: string,
    public readonly count: number,
  ) {}

  public get name() {
    return this._name;
  }

  public get color() {
    return this._color;
  }

  public static createLayers = (data: LayerData[]) => {
    const layers: Record<string, Layer> = {};
    for (const layer of data) {
      layers[layer.id] = new Layer(layer.id, layer.name, layer.color, layer.count);
    }
    return layers;
  };

  public async setColor(color: string) {
    this._color = color;
    await set_layer_color(this.id, color);
  }

  public async setName(name: string) {
    this._name = name;
    await set_layer_name(this.id, name);
  }
}
