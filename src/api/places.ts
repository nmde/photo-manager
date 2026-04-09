import { invoke } from '@tauri-apps/api/core';
import { Layer } from '@/classes/Layer';
import { Place, type PlaceData } from '@/classes/Place';
import { Shape, type ShapeData, type ShapeType } from '@/classes/Shape';

export async function get_layers() {
  return Layer.createLayers(await invoke<Layer[]>('get_layers'));
}

export async function create_layer(id: string, name: string, color: string) {
  return await invoke('create_layer', { id, name, color });
}

export async function set_layer_name(layer: string, value: string) {
  return await invoke('set_layer_name', { layer, value });
}

export async function set_layer_color(layer: string, value: string) {
  return await invoke('set_layer_color', { layer, value });
}

export async function delete_layer(layer: string, recursive: boolean, newLayer?: string) {
  return await invoke('delete_layer', { layer, recursive, newLayer: newLayer ?? null });
}

export async function create_place(
  id: string,
  name: string,
  lat: number,
  lng: number,
  layer: string,
  category: string,
) {
  return await invoke('create_place', { id, name, lat, lng, layer, category });
}

export async function get_places() {
  return Place.createPlaces(await invoke<PlaceData[]>('get_places'));
}

export async function set_place_name(place: string, value: string) {
  return await invoke('set_place_name', { place, value });
}

export async function set_place_category(place: string, value: string) {
  return await invoke('set_place_category', { place, value });
}

export async function set_place_shape(place: string, value: string) {
  return await invoke('set_place_shape', { place, value });
}

export async function set_place_layer(place: string, layer: string) {
  return await invoke('set_place_layer', { place, layer });
}

export async function set_place_position(place: string, lat: number, lng: number) {
  return await invoke('set_place_position', { place, lat, lng });
}

export const delete_place = async (place: string) => await invoke('delete_place', { place });

export const get_shapes = async () => Shape.createShapes(await invoke<ShapeData[]>('get_shapes'));

export async function create_shape(
  id: string,
  shapeType: ShapeType,
  points: string,
  layer: string,
  name: string,
) {
  return await invoke('create_shape', { id, shapeType, points, layer, name });
}

export async function set_shape_points(shape: string, value: string) {
  return await invoke('set_shape_points', { shape, value });
}

export async function set_shape_layer(shape: string, value: string) {
  return await invoke('set_shape_layer', { shape, value });
}

export async function set_shape_name(shape: string, value: string) {
  return await invoke('set_shape_name', { shape, value });
}

export const delete_shape = async (shape: string) => await invoke('delete_shape', { shape });
