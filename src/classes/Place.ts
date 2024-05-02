import { Entity } from './Entity';
import { locToString, type PlaceType } from './Map';

type PlaceData = {
  name: string;
  lat: number;
  lng: number;
  layer: string;
  category: PlaceType;
  shape: string;
};

export class Place extends Entity<PlaceData> {
  public constructor(data: PlaceData) {
    super('Place', data);
  }

  public get posObj() {
    return { lat: this.data.lat, lng: this.data.lng };
  }

  public get pos() {
    return locToString(this.posObj);
  }
}
