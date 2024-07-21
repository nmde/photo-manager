import * as turf from '@turf/turf';
import { Entity } from './Entity';
import type { Position } from './Map';

export type ShapeType = 'polygon' | 'line';

type ShapeData = {
  type: ShapeType;
  points: string;
  layer: string;
  name: string;
};

export class Shape extends Entity<ShapeData> {
  public constructor(data: ShapeData) {
    super('Shape', data);
  }

  public get points() {
    return JSON.parse(this.data.points);
  }

  public set points(points: Position[]) {
    this.data.points = JSON.stringify(points);
  }

  public get area() {
    return 0;
  }
}
