import { set_layer_color, set_layer_name } from '@/api/places';

export type LayerData = {
  id: string;
  name: string;
  color: string;
  count: number;
};

export type LayerRec = Record<LayerData['id'], Layer>;

export class Layer implements LayerData {
  public constructor(
    public readonly id: LayerData['id'],
    public _name: LayerData['name'],
    public _color: LayerData['color'],
    public readonly count: LayerData['count'],
  ) {}

  public get name() {
    return this._name;
  }

  public get color() {
    return this._color;
  }

  public static createLayers = (data: LayerData[]) => {
    const layers: LayerRec = {};
    for (const layer of data) {
      layers[layer.id] = new Layer(layer.id, layer.name, layer.color, layer.count);
    }
    return layers;
  };

  public async setColor(color: LayerData['color']) {
    this._color = color;
    await set_layer_color(this.id, color);
  }

  public async setName(name: LayerData['name']) {
    this._name = name;
    await set_layer_name(this.id, name);
  }
}
