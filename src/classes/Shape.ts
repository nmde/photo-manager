import { set_shape_str } from '@/api/places';
import type { Position } from './Map';

export type ShapeType = 'polygon' | 'line';

export type ShapeData = {
  id: string;
  shape_type: ShapeType;
  points: string;
  layer: string;
  name: string;
};

export class Shape {
  public constructor(
    public readonly id: string,
    public readonly type: ShapeType,
    public _points: string,
    public _layer: string,
    public _name: string,
  ) {}

  public get points() {
    return JSON.parse(this._points) as Position[];
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
    const shapes: Record<string, Shape> = {};
    for (const shape of data.map(
      ({ id, shape_type, points, layer, name }) => new Shape(id, shape_type, points, layer, name),
    )) {
      shapes[shape.id] = shape;
    }
    return shapes;
  }

  public async setPoints(points: Position[]) {
    this._points = JSON.stringify(points);
    await set_shape_str(this.id, 'points', this._points);
  }

  public async setLayer(layer: string) {
    this._layer = layer;
    await set_shape_str(this.id, 'layer', layer);
  }

  public async setName(name: string) {
    this._name = name;
    await set_shape_str(this.id, 'name', name);
  }
}
