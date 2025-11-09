import type { Position } from './Map';
import { invoke } from '@tauri-apps/api/core';

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
    private _id: string,
    private _type: ShapeType,
    private _points: string,
    private _layer: string,
    private _name: string,
  ) {}

  public get id() {
    return this._id;
  }

  public get type() {
    return this._type;
  }

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
    await invoke('set_shape_str', {
      shape: this._id,
      property: 'points',
      value: this._points,
    });
  }

  public async setLayer(layer: string) {
    this._layer = layer;
    await invoke('set_shape_str', {
      shape: this._id,
      property: 'layer',
      value: layer,
    });
  }

  public async setName(name: string) {
    this._name = name;
    await invoke('set_shape_str', {
      shape: this._id,
      property: 'name',
      value: name,
    });
  }
}
