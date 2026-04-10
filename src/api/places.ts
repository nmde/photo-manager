import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Layer, type LayerData } from '@/classes/Layer';
import { Place, type PlaceData } from '@/classes/Place';
import { Shape, type ShapeData, type ShapeType } from '@/classes/Shape';

export function get_layers() {
  return new APIResult<LayerData[], Record<string, Layer>>(
    async () => await invoke('get_layers'),
    l => Layer.createLayers(l),
  );
}

export function get_places() {
  return new APIResult<PlaceData[], Record<string, Place>>(
    async () => await invoke('get_places'),
    places => Place.createPlaces(places),
  );
}

export function get_shapes() {
  return new APIResult<ShapeData[], Record<string, Shape>>(
    async () => await invoke('get_shapes'),
    shapes => Shape.createShapes(shapes),
  );
}

export async function create_layer(id: string, name: string, color: string) {
  await invoke('create_layer', { id, name, color });
}

export async function set_layer_name(layer: string, value: string) {
  await invoke('set_layer_name', { layer, value });
}

export async function set_layer_color(layer: string, value: string) {
  await invoke('set_layer_color', { layer, value });
}

export async function delete_layer(layer: string, recursive: boolean, newLayer?: string) {
  await invoke('delete_layer', { layer, recursive, newLayer: newLayer ?? null });
}

export async function create_place(
  id: string,
  name: string,
  lat: number,
  lng: number,
  layer: string,
  category: string,
) {
  await invoke('create_place', { id, name, lat, lng, layer, category });
}

export async function set_place_name(place: string, value: string) {
  await invoke('set_place_name', { place, value });
}

export async function set_place_category(place: string, value: string) {
  await invoke('set_place_category', { place, value });
}

export async function set_place_shape(place: string, value: string | null) {
  await invoke('set_place_shape', { place, value });
}

export async function set_place_layer(place: string, layer: string) {
  await invoke('set_place_layer', { place, layer });
}

export async function set_place_position(place: string, lat: number, lng: number) {
  await invoke('set_place_position', { place, lat, lng });
}

export async function delete_place(place: string) {
  await invoke('delete_place', { place });
}

export async function create_shape(
  id: string,
  shapeType: ShapeType,
  points: string,
  layer: string,
  name: string,
) {
  await invoke('create_shape', { id, shapeType, points, layer, name });
}

export async function set_shape_points(shape: string, value: string) {
  await invoke('set_shape_points', { shape, value });
}

export async function set_shape_layer(shape: string, value: string) {
  await invoke('set_shape_layer', { shape, value });
}

export async function set_shape_name(shape: string, value: string) {
  await invoke('set_shape_name', { shape, value });
}

export async function delete_shape(shape: string) {
  await invoke('delete_shape', { shape });
}
