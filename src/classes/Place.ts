import { invoke } from '@tauri-apps/api/core';
import { locToString, type PlaceType, type Position } from './Map';

export type PlaceData = {
  id: string;
  name: string;
  lat: number;
  lng: number;
  layer: string;
  category: PlaceType;
  shape: string;
  tags: string;
  notes: string;
  count: number;
};

export class Place {
  public isNewestPlace = false;

  public constructor(
    private _id: string,
    private _name: string,
    private lat: number,
    private lng: number,
    private _layer: string,
    private _category: PlaceType,
    private _shape: string,
    private _tags: string,
    private _notes: string,
    public count: number,
  ) {}

  public get id() {
    return this._id;
  }

  public get name() {
    return this._name;
  }

  public get posObj() {
    return { lat: this.lat, lng: this.lng };
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

  public get tags() {
    return this._tags.length === 0 ? [] : this._tags.split(',');
  }

  public get notes() {
    return this._notes;
  }

  public static createPlaces(data: Record<string, PlaceData>) {
    const places: Record<string, Place> = {};
    for (const place of Object.values(data).map(
      ({ id, name, lat, lng, layer, category, shape, tags, notes, count }) =>
        new Place(id, name, lat, lng, layer, category, shape, tags, notes, count),
    )) {
      places[place.id] = place;
    }
    return places;
  }

  public async setName(name: string) {
    this._name = name;
    await invoke('set_place_str', {
      place: this.id,
      property: 'name',
      value: name,
    });
  }

  public async setPosition(position: Position) {
    this.lat = position.lat;
    this.lng = position.lng;
    await invoke('set_place_position', position);
  }

  public async setCategory(category: PlaceType) {
    this._category = category;
    await invoke('set_place_str', {
      place: this.id,
      property: 'category',
      value: category,
    });
  }

  public async setShape(shape: string) {
    this._shape = shape;
    await invoke('set_place_str', {
      place: this.id,
      property: 'shape',
      value: shape,
    });
  }

  public async setTags(tags: string[]) {
    this._tags = tags.join(',');
    await invoke('set_place_str', {
      place: this.id,
      property: 'tags',
      value: this._tags,
    });
  }

  public async setLayer(layer: string) {
    this._layer = layer;
    await invoke('set_place_str', {
      place: this.id,
      property: 'layer',
      value: layer,
    });
  }

  public async setNotes(notes: string) {
    this._notes = notes;
    await invoke('set_place_str', {
      place: this.id,
      property: 'notes',
      value: notes,
    });
  }
}
