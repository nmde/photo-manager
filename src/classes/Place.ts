import type { LayerData } from './Layer';
import type { ShapeData } from './Shape';
import type { Nullable } from '@/types';
import {
  set_place_category,
  set_place_layer,
  set_place_name,
  set_place_position,
  set_place_shape,
} from '@/api/places';
import { locToString, type PlaceType, type Position } from './Map';
import { SortableItem } from './SortableItem';

export type PlaceData = {
  id: string;
  name: string;
  lat: number;
  lng: number;
  layer: LayerData['id'];
  category: PlaceType;
  shape: Nullable<ShapeData['id']>;
  count: number;
};

export type PlaceRec = Record<PlaceData['id'], Place>;

export class Place extends SortableItem implements PlaceData {
  public constructor(
    public readonly id: PlaceData['id'],
    public _name: PlaceData['name'],
    public _lat: PlaceData['lat'],
    public _lng: PlaceData['lng'],
    public _layer: PlaceData['layer'],
    public _category: PlaceData['category'],
    public _shape: PlaceData['shape'],
    public count: PlaceData['count'],
  ) {
    super(id, count, _name, null);
  }

  public get lat() {
    return this._lat;
  }

  public get lng() {
    return this._lng;
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

  public static createPlaces(data: PlaceData[]) {
    const places: PlaceRec = {};
    for (const place of data.map(
      ({ id, name, lat, lng, layer, category, shape, count }) =>
        new Place(id, name, lat, lng, layer, category, shape, count),
    )) {
      places[place.id] = place;
    }
    return places;
  }

  public async setName(name: PlaceData['name']) {
    this._name = name;
    await set_place_name(this.id, name);
  }

  public async setPosition(position: Position) {
    this._lat = position.lat;
    this._lng = position.lng;
    await set_place_position(this.id, position.lat, position.lng);
  }

  public async setCategory(category: PlaceData['category']) {
    this._category = category;
    await set_place_category(this.id, category);
  }

  public async setShape(shape: PlaceData['shape']) {
    this._shape = shape;
    await set_place_shape(this.id, shape);
  }

  public async setLayer(layer: PlaceData['layer']) {
    this._layer = layer;
    await set_place_layer(this.id, layer);
  }
}
