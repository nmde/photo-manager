import type { Position } from './Map';
import { Entity } from './Entity';

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
    return JSON.parse(this.data.points) as Position[];
  }

  public get area() {
    return 0;
  }

  public set points(points: Position[]) {
    this.data.points = JSON.stringify(points);
  }
}
