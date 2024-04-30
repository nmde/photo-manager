import { Entity } from './Entity';

type LayerData = {
  name: string;
  color: string;
};

export class Layer extends Entity<LayerData> {
  public constructor(data: LayerData) {
    super('Layer', data);
  }
}
