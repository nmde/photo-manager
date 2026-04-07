import { set_layer_str } from '@/api/places';

export type LayerData = {
  id: string;
  name: string;
  color: string;
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

  public static createLayers = (data: Record<string, LayerData>) => {
    const layers: Record<string, Layer> = {};
    for (const layer in data) {
      layers[layer] = new Layer(
        layer,
        data[layer]?.name ?? '',
        data[layer]?.color ?? '',
        data[layer]?.count ?? 0,
      );
    }
    return layers;
  };

  public async setColor(color: string) {
    this._color = color;
    await set_layer_str(this.id, 'color', color);
  }

  public async setName(name: string) {
    this._name = name;
    await set_layer_str(this.id, 'name', name);
  }
}
