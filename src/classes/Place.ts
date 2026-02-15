import { set_place_layer, set_place_position, set_place_str } from '@/api/places';
import { locToString, type PlaceType, type Position } from './Map';

export type PlaceData = {
  id: string;
  name: string;
  lat: number;
  lng: number;
  layer: string;
  category: PlaceType;
  shape: string;
  count: number;
};

export class Place {
  public constructor(
    public readonly id: string,
    public _name: string,
    public _lat: number,
    public _lng: number,
    public _layer: string,
    public _category: PlaceType,
    public _shape: string,
    public count: number,
  ) {}

  public get name() {
    return this._name;
  }

  public get posObj() {
    return { lat: this._lat, lng: this._lng };
  }

  public get pos() {
    return locToString(this.posObj);
  }

  public get layer() {
    return this._layer;
  }

  public get shape() {
    return this._shape;
  }

  public get category() {
    return this._category;
  }

  public static createPlaces(data: Record<string, PlaceData>) {
    const places: Record<string, Place> = {};
    for (const place of Object.values(data).map(
      ({ id, name, lat, lng, layer, category, shape, count }) =>
        new Place(id, name, lat, lng, layer, category, shape, count),
    )) {
      places[place.id] = place;
    }
    return places;
  }

  public async setName(name: string) {
    this._name = name;
    await set_place_str(this.id, 'name', name);
  }

  public async setPosition(position: Position) {
    this._lat = position.lat;
    this._lng = position.lng;
    await set_place_position(this.id, position.lat, position.lng);
  }

  public async setCategory(category: PlaceType) {
    this._category = category;
    await set_place_str(this.id, 'category', category);
  }

  public async setShape(shape: string) {
    this._shape = shape;
    await set_place_str(this.id, 'shape', shape);
  }

  public async setLayer(layer: string) {
    this._layer = layer;
    await set_place_layer(this.id, layer);
  }
}
