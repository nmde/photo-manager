import { Layer, type LayerData } from '@/classes/Layer';
import { Place, type PlaceData } from '@/classes/Place';
import { Shape, type ShapeData, type ShapeType } from '@/classes/Shape';
import { invoke } from '@tauri-apps/api/core';

export const get_layers = async () =>
  Layer.createLayers(await invoke<Record<string, Layer>>('get_layers'));

export const create_layer = async (id: string, name: string, color: string) =>
  await invoke('create_layer', { id, name, color });

export const set_layer_str = async (layer: string, property: string, value: string) =>
  await invoke('set_layer_str', { layer, property, value });

export const delete_layer = async (layer: string, recursive: boolean, newLayer?: string) =>
  await invoke('delete_layer', { layer, recursive, newLayer: newLayer ?? null });

export const create_place = async (
  id: string,
  name: string,
  lat: number,
  lng: number,
  layer: string,
  category: string,
) => await invoke('create_place', { id, name, lat, lng, layer, category });

export const get_places = async () =>
  Place.createPlaces(await invoke<Record<string, PlaceData>>('get_places'));

export const set_place_name = async (place: string, value: string) =>
  await invoke('set_place_name', { place, value });

export const set_place_category = async (place: string, value: string) =>
  await invoke('set_place_category', { place, value });

export const set_place_shape = async (place: string, value: string) =>
  await invoke('set_place_shape', { place, value });

export const set_place_layer = async (place: string, layer: string) =>
  await invoke('set_place_layer', { place, layer });

export const set_place_position = async (place: string, lat: number, lng: number) =>
  await invoke('set_place_position', { place, lat, lng });

export const delete_place = async (place: string) => await invoke('delete_place', { place });

export const get_shapes = async () => Shape.createShapes(await invoke<ShapeData[]>('get_shapes'));

export const create_shape = async (
  id: string,
  shapeType: ShapeType,
  points: string,
  layer: string,
  name: string,
) => await invoke('create_shape', { id, shapeType, points, layer, name });

export const set_shape_points = async (shape: string, value: string) =>
  await invoke('set_shape_points', { shape, value });

export const set_shape_layer = async (shape: string, value: string) =>
  await invoke('set_shape_layer', { shape, value });

export const set_shape_name = async (shape: string, value: string) =>
  await invoke('set_shape_name', { shape, value });

export const delete_shape = async (shape: string) => await invoke('delete_shape', { shape });
