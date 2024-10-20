import { Entity } from './Entity';

type CameraData = {
  name: string;
};

export class Camera extends Entity<CameraData> {
  public count = 0;

  public constructor(data: CameraData) {
    super('Camera', data);
  }
}
