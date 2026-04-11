import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Layer, type LayerData, type LayerRec } from '@/classes/Layer';
import { Place, type PlaceData, type PlaceRec } from '@/classes/Place';
import { Shape, type ShapeData, type ShapeRec } from '@/classes/Shape';

export function get_layers() {
  return new APIResult<LayerData[], LayerRec>(
    async () => await invoke('get_layers'),
    l => Layer.createLayers(l),
  );
}

export function get_places() {
  return new APIResult<PlaceData[], PlaceRec>(
    async () => await invoke('get_places'),
    places => Place.createPlaces(places),
  );
}

export function get_shapes() {
  return new APIResult<ShapeData[], ShapeRec>(
    async () => await invoke('get_shapes'),
    shapes => Shape.createShapes(shapes),
  );
}

export async function create_layer(
  id: LayerData['id'],
  name: LayerData['name'],
  color: LayerData['color'],
) {
  await invoke('create_layer', { id, name, color });
}

export async function set_layer_name(layer: LayerData['id'], value: LayerData['name']) {
  await invoke('set_layer_name', { layer, value });
}

export async function set_layer_color(layer: LayerData['id'], value: LayerData['color']) {
  await invoke('set_layer_color', { layer, value });
}

export async function delete_layer(
  layer: LayerData['id'],
  recursive: boolean,
  newLayer?: LayerData['id'],
) {
  await invoke('delete_layer', { layer, recursive, newLayer: newLayer ?? null });
}

export async function create_place(
  id: PlaceData['id'],
  name: PlaceData['name'],
  lat: PlaceData['lat'],
  lng: PlaceData['lng'],
  layer: PlaceData['layer'],
  category: PlaceData['category'],
) {
  await invoke('create_place', { id, name, lat, lng, layer, category });
}

export async function set_place_name(place: PlaceData['id'], value: PlaceData['name']) {
  await invoke('set_place_name', { place, value });
}

export async function set_place_category(place: PlaceData['id'], value: PlaceData['category']) {
  await invoke('set_place_category', { place, value });
}

export async function set_place_shape(place: PlaceData['id'], value: PlaceData['shape']) {
  await invoke('set_place_shape', { place, value });
}

export async function set_place_layer(place: PlaceData['id'], layer: PlaceData['layer']) {
  await invoke('set_place_layer', { place, layer });
}

export async function set_place_position(
  place: PlaceData['id'],
  lat: PlaceData['lat'],
  lng: PlaceData['lng'],
) {
  await invoke('set_place_position', { place, lat, lng });
}

export async function delete_place(place: PlaceData['id']) {
  await invoke('delete_place', { place });
}

export async function create_shape(
  id: ShapeData['id'],
  shapeType: ShapeData['shape_type'],
  points: ShapeData['points'],
  layer: ShapeData['layer'],
  name: ShapeData['name'],
) {
  await invoke('create_shape', { id, shapeType, points, layer, name });
}

export async function set_shape_points(shape: ShapeData['id'], value: ShapeData['points']) {
  await invoke('set_shape_points', { shape, value });
}

export async function set_shape_layer(shape: ShapeData['id'], value: ShapeData['layer']) {
  await invoke('set_shape_layer', { shape, value });
}

export async function set_shape_name(shape: ShapeData['id'], value: ShapeData['name']) {
  await invoke('set_shape_name', { shape, value });
}

export async function delete_shape(shape: ShapeData['id']) {
  await invoke('delete_shape', { shape });
}
