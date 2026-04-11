import type { LayerData } from './Layer';
import type { Position } from './Map';
import { set_shape_layer, set_shape_name, set_shape_points } from '@/api/places';

export type ShapeType = 'polygon' | 'line';

export type ShapeData = {
  id: string;
  shape_type: ShapeType;
  points: string;
  layer: LayerData['id'];
  name: string;
};

export type ShapeRec = Record<ShapeData['id'], Shape>;

export class Shape implements ShapeData {
  public constructor(
    public readonly id: ShapeData['id'],
    public readonly type: ShapeData['shape_type'],
    public _points: ShapeData['points'],
    public _layer: ShapeData['layer'],
    public _name: ShapeData['name'],
  ) {}

  public get points() {
    return this._points;
  }

  public get shape() {
    return JSON.parse(this._points) as Position[];
  }

  public get shape_type() {
    return this.type;
  }

  public get layer() {
    return this._layer;
  }

  public get name() {
    return this._name;
  }

  public get area() {
    return 0;
  }

  public static createShapes(data: ShapeData[]) {
    const shapes: ShapeRec = {};
    for (const shape of data.map(
      ({ id, shape_type, points, layer, name }) => new Shape(id, shape_type, points, layer, name),
    )) {
      shapes[shape.id] = shape;
    }
    return shapes;
  }

  public async setPoints(points: Position[]) {
    this._points = JSON.stringify(points);
    await set_shape_points(this.id, this._points);
  }

  public async setLayer(layer: ShapeData['layer']) {
    this._layer = layer;
    await set_shape_layer(this.id, layer);
  }

  public async setName(name: ShapeData['name']) {
    this._name = name;
    await set_shape_name(this.id, name);
  }
}
