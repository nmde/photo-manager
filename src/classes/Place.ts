import { Entity } from './Entity';
import { locToString } from './Map';

type PlaceData = {
  name: string;
  lat: number;
  lng: number;
  layer: string;
};

export class Place extends Entity<PlaceData> {
  public constructor(data: PlaceData) {
    super('Place', data);
  }

  public get pos() {
    return locToString({ lat: this.data.lat, lng: this.data.lng });
  }
}
