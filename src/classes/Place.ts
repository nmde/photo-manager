import { Entity } from './Entity';
import { locToString, type PlaceType } from './Map';

type PlaceData = {
  name: string;
  lat: number;
  lng: number;
  layer: string;
  category: PlaceType;
  shape: string;
  tags: string;
  notes: string;
};

export class Place extends Entity<PlaceData> {
  public count = 0;

  public constructor(data: PlaceData) {
    super('Place', data);
  }

  public get posObj() {
    return { lat: this.data.lat, lng: this.data.lng };
  }

  public get pos() {
    return locToString(this.posObj);
  }

  public get tags() {
    if (this.data.tags.length === 0) {
      return [];
    }
    return this.data.tags.split(',');
  }

  public set tags(tags: string[]) {
    this.data.tags = tags.join(',');
  }
}
