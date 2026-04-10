<script setup lang="ts">
  import type { Layer } from '@/classes/Layer';
  import { v4 as uuid } from 'uuid';
  import {
    create_place,
    create_shape,
    delete_place,
    delete_shape,
    get_layers,
    get_places,
    get_shapes,
  } from '@/api/places';
  import { icons, locToString, Map, type PlaceType, type Position } from '@/classes/Map';
  import { Place } from '@/classes/Place';
  import { Shape, type ShapeType } from '@/classes/Shape';
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const { reportError } = store;
  const router = useRouter();

  const createDialog = ref(false);
  const position = ref<Position>({ lat: 0, lng: 0 });
  const mapEl = ref<HTMLDivElement>();
  const newPlaceMapEl = ref<HTMLDivElement>();
  const placeName = ref('');
  const placeCategory = ref<keyof typeof icons>('hospital');
  const mapInitialized = ref(false);
  const layers = ref<Record<string, Layer>>({});
  const targetLayer = ref('');
  const drawMode = ref(false);
  const tmpShape = ref<Position[]>([]);
  const tmpShapeType = ref<ShapeType>('line');
  const placeMap = ref<Record<string, Place[]>>({});
  const shapeMap = ref<Record<string, Shape[]>>({});
  const shapeName = ref('');
  const shapeDialog = ref(false);
  const targetPlace = ref('');
  const editingShape = ref(false);
  const targetShape = ref('');
  const hideMarkers = ref(false);
  const hideLabels = ref(false);
  const changeLayerDialog = ref(false);
  const changeShapeLayerDialog = ref(false);
  const places = ref<Record<string, Place>>({});
  const shapes = ref<Record<string, Shape>>({});

  const categories = computed(() => Object.keys(icons));

  const sortedLayers = computed(() =>
    Object.values(layers.value).toSorted((a, b) => a.name.localeCompare(b.name)),
  );

  const map = new Map();
  const newPlaceMap = new Map();
  let lastChangedIndex = -1;

  function openCreateDialog(layer: string) {
    targetLayer.value = layer;
    createDialog.value = true;
    if (mapInitialized.value) {
      newPlaceMap.clearMarkers();
    } else {
      setTimeout(async () => {
        await newPlaceMap.initialize(newPlaceMapEl.value as unknown as HTMLElement);
        newPlaceMap.on('dblclick', pos => {
          position.value = pos;
          newPlaceMap.createMarker(
            locToString(pos),
            '0',
            placeCategory.value,
            layers.value[targetLayer.value]?.color,
            placeName.value,
          );
        });
      }, 500);
    }
  }

  function setShapeColor(shape: Shape, color: string) {
    map.removeShape(shape.id);
    map.createShape(shape.type, shape.points, color, shape.id, false);
  }

  function setPlaceColor(place: Place, color: string) {
    map.removeMarker(place.id);
    map.createMarker(place.pos, place.id, place.category, color, place.name, place.count);
    // update color of linked polygons
    if (place.shape !== null) {
      const s = shapes.value[place.shape];
      if (place.shape.length > 0 && s) {
        setShapeColor(s, color);
      }
    }
  }

  async function setLayerColor(layer: Layer, color: string) {
    await layers.value[layer.id]?.setColor(color);
    for (const place of placeMap.value[layer.id] ?? []) {
      setPlaceColor(place, color);
    }
    for (const shape of shapeMap.value[layer.id] ?? []) {
      setShapeColor(shape, color);
    }
  }

  async function deleteShapeFunc(layer_id: string, id: string) {
    await delete_shape(id);
    map.removeShape(id);
    shapeMap.value[layer_id]?.splice(
      shapeMap.value[layer_id].findIndex(p => p.id === id),
      1,
    );
  }

  async function createShape(type: ShapeType, points: Position[], layer: string, name: string) {
    const id = uuid();
    await create_shape(id, type, JSON.stringify(points), layer, name);
    const shape = new Shape(id, type, JSON.stringify(points), layer, name);
    shapes.value[id] = shape;
    return shape;
  }

  async function createPlace(name: string, position: Position, layer: string, category: PlaceType) {
    const id = uuid();
    await create_place(id, name, position.lat, position.lng, layer, category);
    const place = new Place(id, name, position.lat, position.lng, layer, category, '', 0);
    places.value[id] = place;
    return place;
  }

  async function deletePlace(place: Place) {
    await delete_place(place.id);
    if (place.shape !== null) {
      deleteShapeFunc(place.layer, place.shape);
    }
    placeMap.value[place.layer]?.splice(
      placeMap.value[place.layer]?.findIndex(p => p.id === place.id) ?? -1,
      1,
    );
    map.removeMarker(place.id);
  }

  function editPlaceShape(place: Place) {
    if (place.shape !== null) {
      const shape = shapes.value[place.shape];
      map.removeShape(place.shape);
      targetLayer.value = place.layer;
      if (shape) {
        tmpShape.value = shape.points;
        editingShape.value = true;
        targetShape.value = shape.id;
        tmpShapeType.value = shape.type;
        map.createShape(
          shape.type,
          shape.points,
          layers.value[shape.layer]?.color ?? '',
          shape.id,
          true,
        );
        drawMode.value = true;
      }
    }
  }

  function moveIcon(place: Place) {
    targetLayer.value = place.layer;
    map.removeMarker(place.id);
    editingShape.value = true;
    const listener = map.on('dblclick', async pos => {
      map.createMarker(
        locToString(pos),
        place.id,
        place.category,
        layers.value[place.layer]?.color,
        place.name,
        place.count,
      );
      map.off('dblclick', listener);
      await places.value[place.id]?.setPosition(pos);
    });
  }

  async function saveShape() {
    if (editingShape.value) {
      await shapes.value[targetShape.value]?.setPoints(tmpShape.value);
      map.removeShape(targetShape.value);
      map.createShape(
        shapes.value[targetShape.value]?.type ?? 'line',
        tmpShape.value,
        layers.value[targetLayer.value]?.color ?? '',
        targetShape.value,
      );
    } else {
      const s = await createShape(
        tmpShapeType.value,
        tmpShape.value,
        targetLayer.value,
        shapeName.value,
      );
      if (targetPlace.value.length > 0) {
        await places.value[targetPlace.value]?.setShape(s.id);
      } else {
        if (!shapeMap.value[targetLayer.value]) {
          shapeMap.value[targetLayer.value] = [];
        }
        shapeMap.value[targetLayer.value]?.push(s);
      }
      map.removeShape(`${prevShape}`);
      map.createShape(
        tmpShapeType.value,
        tmpShape.value,
        layers.value[targetLayer.value]?.color ?? '',
        s.id,
      );
    }
    prevShape = 0;
    drawMode.value = false;
  }

  async function splitShape() {
    if (lastChangedIndex > 0 && lastChangedIndex < tmpShape.value.length - 1) {
      const split1 = tmpShape.value.slice(0, lastChangedIndex);
      const split2 = tmpShape.value.slice(lastChangedIndex + 1);
      await shapes.value[targetShape.value]?.setPoints(split1);
      map.removeShape(targetShape.value);
      map.createShape(
        shapes.value[targetShape.value]?.type ?? 'line',
        split1,
        layers.value[targetLayer.value]?.color ?? '',
        targetShape.value,
      );
      const newShape = await createShape(
        tmpShapeType.value,
        split2,
        targetLayer.value,
        `${shapes.value[targetShape.value]?.name} - Split`,
      );
      if (!shapeMap.value[targetLayer.value]) {
        shapeMap.value[targetLayer.value] = [];
      }
      shapeMap.value[targetLayer.value]?.push(newShape);
      map.createShape(
        tmpShapeType.value,
        split2,
        layers.value[targetLayer.value]?.color ?? '',
        newShape.id,
      );
      drawMode.value = false;
    }
  }

  let prevShape = 0;
  onMounted(async () => {
    await get_layers()
      .ok(l => (layers.value = l))
      .err(msg => reportError(msg))
      .send();
    await get_places()
      .ok(p => (places.value = p))
      .err(msg => reportError(msg))
      .send();
    await get_shapes()
      .ok(s => (shapes.value = s))
      .err(msg => reportError(msg))
      .send();
    placeMap.value = {};
    shapeMap.value = {};
    for (const id in layers.value) {
      placeMap.value[id] = [];
      shapeMap.value[id] = [];
    }
    if (mapEl.value) {
      await map.initialize(mapEl.value);
    }
    const linkedShapes: string[] = [];
    for (const id in layers.value) {
      for (const place of Object.values(places.value).filter(place => place.layer === id)) {
        if (!placeMap.value[place.layer]) {
          placeMap.value[place.layer] = [];
        }
        placeMap.value[place.layer]?.push(place);
        if (place.shape !== null) {
          linkedShapes.push(place.shape);
        }
        map.createMarker(
          place.pos,
          place.id,
          place.category,
          layers.value[place.layer]?.color,
          place.name,
          place.count,
        );
      }
      for (const shape of Object.values(shapes.value).filter(shape => shape.layer === id)) {
        if (!linkedShapes.includes(shape.id)) {
          if (!shapeMap.value[shape.layer]) {
            shapeMap.value[shape.layer] = [];
          }
          shapeMap.value[shape.layer]?.push(shape);
        }
        map.createShape(shape.type, shape.points, layers.value[shape.layer]?.color ?? '', shape.id);
      }
    }
    map.on('click', pos => {
      if (drawMode.value) {
        tmpShape.value.push(pos);
        let nextId = (prevShape + 1).toString();
        if (editingShape.value) {
          map.removeShape(targetShape.value);
          nextId = targetShape.value;
        } else if (prevShape > 0) {
          map.removeShape(prevShape.toString());
        }
        prevShape += 1;
        map.createShape(
          tmpShapeType.value,
          tmpShape.value,
          layers.value[targetLayer.value]?.color ?? '',
          nextId,
          true,
        );
      }
    });
    map.on('shapeUpdate', pos => {
      if (drawMode.value) {
        for (const [i, item] of pos.getArray().entries()) {
          const x = item.toJSON();
          if (locToString(tmpShape.value[i]) !== locToString(x)) {
            lastChangedIndex = i;
          }
          tmpShape.value[i] = x;
        }
      }
    });
    map.on(
      'markerClicked',
      place =>
        void (async place => {
          await router.push(`/tagger?place=${place}`);
        })(place),
    );
  });
</script>

<template>
  <v-container fluid>
    <v-row>
      <v-col class="layers" cols="4">
        <v-expansion-panels>
          <v-expansion-panel
            v-for="layer in sortedLayers"
            :key="layer.id"
            :title="`${layer.name} (${layer.count})`"
          >
            <v-expansion-panel-text>
              {{ layer.name }}
              <v-menu :disabled="drawMode">
                <template #activator="{ props }">
                  <v-btn flat icon v-bind="props">
                    <v-icon>mdi-plus</v-icon>
                  </v-btn>
                </template>
                <v-list>
                  <v-list-item @click="openCreateDialog(layer.id)">Add Place</v-list-item>
                  <v-list-item
                    @click="
                      () => {
                        tmpShapeType = 'line';
                        targetLayer = layer.id;
                        targetPlace = '';
                        shapeName = '';
                        shapeDialog = true;
                        editingShape = false;
                      }
                    "
                  >
                    Add Line
                  </v-list-item>
                  <v-list-item
                    @click="
                      () => {
                        tmpShapeType = 'polygon';
                        targetLayer = layer.id;
                        targetPlace = '';
                        shapeName = '';
                        shapeDialog = true;
                        editingShape = false;
                      }
                    "
                  >
                    Add Polygon
                  </v-list-item>
                </v-list>
              </v-menu>
              <color-picker
                :color="layer.color"
                @update="async color => setLayerColor(layer, color)"
              />
              <v-expansion-panels>
                <v-expansion-panel v-for="place in placeMap[layer.id]" :key="place.id">
                  <v-expansion-panel-title>
                    <v-icon>{{ icons[place.category] }}</v-icon>
                    &nbsp;{{ place.name }}
                  </v-expansion-panel-title>
                  <v-expansion-panel-text>
                    <v-btn
                      v-if="place.shape === null"
                      color="primary"
                      @click="
                        () => {
                          tmpShape = [];
                          drawMode = true;
                          editingShape = false;
                          tmpShapeType = 'polygon';
                          targetLayer = layer.id;
                          targetPlace = place.id;
                        }
                      "
                    >
                      Draw Polygon
                    </v-btn>
                    <div v-if="place.shape !== null">
                      <v-btn @click="editPlaceShape(place)">Edit Polygon</v-btn>
                      Area: {{ shapes[place.shape]?.area }}
                    </div>
                    <v-menu>
                      <template #activator="{ props }">
                        <v-btn flat icon v-bind="props">
                          <v-icon>mdi-menu</v-icon>
                        </v-btn>
                      </template>
                      <v-list>
                        <v-list-item
                          @click="
                            () => {
                              map.setCenter(place.posObj.lat, place.posObj.lng);
                            }
                          "
                        >
                          Find On Map
                        </v-list-item>
                        <v-list-item @click="deletePlace(place)">Delete Place</v-list-item>
                        <v-list-item
                          @click="
                            () => {
                              targetPlace = place.id;
                              changeLayerDialog = true;
                            }
                          "
                        >
                          Change Layer
                        </v-list-item>
                        <v-list-item @click="moveIcon(place)">Move Icon</v-list-item>
                        <v-list-item
                          @click="
                            () => {
                              router.push(`/tagger?place=${place.id}`);
                            }
                          "
                        >
                          View Photos
                        </v-list-item>
                      </v-list>
                    </v-menu>
                    <v-text-field
                      v-model="place.name"
                      label="Name"
                      @update:model-value="
                        async () => {
                          await places[place.id]?.setName(place.name);
                        }
                      "
                    />
                    <v-select
                      v-model="place.category"
                      :items="categories"
                      @update:model-value="
                        async () => {
                          await places[place.id]?.setCategory(place.category);
                          map.removeMarker(place.id);
                          map.createMarker(
                            place.pos,
                            place.id,
                            place.category,
                            layers[place.layer]?.color,
                            place.name,
                            place.count,
                          );
                        }
                      "
                    />
                  </v-expansion-panel-text>
                </v-expansion-panel>
                <v-expansion-panel v-for="shape in shapeMap[layer.id]" :key="shape.id">
                  <v-expansion-panel-title>
                    <v-icon v-if="shape.type === 'line'">mdi-vector-line</v-icon>
                    <v-icon v-else>mdi-vector-polygon</v-icon>
                    &nbsp;{{ shape.name }}
                  </v-expansion-panel-title>
                  <v-expansion-panel-text>
                    <v-text-field
                      v-model="shape.name"
                      label="Name"
                      @update:model-value="
                        async () => {
                          await shapes[shape.id]?.setName(shape.name);
                        }
                      "
                    />
                    <v-btn
                      @click="
                        () => {
                          map.removeShape(shape.id);
                          targetLayer = layer.id;
                          tmpShape = shape.points;
                          editingShape = true;
                          targetShape = shape.id;
                          tmpShapeType = shape.type;
                          map.createShape(
                            shape.type,
                            shape.points,
                            layers[shape.layer]?.color ?? '',
                            shape.id,
                            true,
                          );
                          drawMode = true;
                        }
                      "
                    >
                      Edit Shape
                    </v-btn>
                    <v-btn
                      @click="
                        () => {
                          targetShape = shape.id;
                          changeShapeLayerDialog = true;
                        }
                      "
                    >
                      Change Layer
                    </v-btn>
                    <v-btn
                      @click="
                        async () => {
                          await deleteShapeFunc(layer.id, shape.id);
                        }
                      "
                    >
                      Delete Shape
                    </v-btn>
                  </v-expansion-panel-text>
                </v-expansion-panel>
              </v-expansion-panels>
            </v-expansion-panel-text>
          </v-expansion-panel>
        </v-expansion-panels>
      </v-col>
      <v-col cols="8">
        <div class="map-container">
          <div ref="mapEl" class="map" />
        </div>
        <v-btn v-if="drawMode" color="primary" @click="saveShape()">Save Shape</v-btn>
        <v-btn v-if="drawMode" color="primary" @click="splitShape()">Splt Shape</v-btn>
        <v-btn
          v-if="drawMode"
          color="error"
          @click="
            () => {
              tmpShape = [];
              map.removeShape(targetShape);
              drawMode = false;
            }
          "
        >
          Cancel Shape
        </v-btn>
        <v-btn
          v-if="!hideMarkers"
          @click="
            () => {
              hideMarkers = true;
              map.hideAllMarkers();
            }
          "
        >
          Hide Markers
        </v-btn>
        <v-btn
          v-if="hideMarkers"
          @click="
            () => {
              hideMarkers = false;
              map.showAllMarkers();
            }
          "
        >
          Show Markers
        </v-btn>
        <v-btn
          v-if="!hideLabels"
          @click="
            () => {
              map.setStyle(Map.BlankMap);
              hideLabels = true;
              if (hideMarkers) {
                map.hideAllMarkers();
              }
            }
          "
        >
          Hide Labels
        </v-btn>
        <v-btn
          v-if="hideLabels"
          @click="
            () => {
              map.setStyle(Map.DefaultMap);
              hideLabels = false;
              if (hideMarkers) {
                map.hideAllMarkers();
              }
            }
          "
        >
          Show Labels
        </v-btn>
      </v-col>
    </v-row>
  </v-container>
  <v-dialog v-model="createDialog">
    <v-card :title="`Add a Location to ${layers[targetLayer]?.name}`">
      <v-card-text>
        <v-text-field v-model="placeName" label="Name" />
        Category: <v-select v-model="placeCategory" :items="categories" />
        <div class="map-container">
          <div ref="newPlaceMapEl" class="map" />
        </div>
        Selected position: {{ position }}<br />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              const p = await createPlace(placeName, position, targetLayer, placeCategory);
              placeMap[targetLayer]?.push(p);
              map.createMarker(
                locToString(position),
                p.id,
                placeCategory,
                layers[targetLayer]?.color,
                placeName,
              );
              createDialog = false;
              placeName = '';
              placeCategory = 'hospital';
              position = { lat: 0, lng: 0 };
            }
          "
        >
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="shapeDialog">
    <v-card :title="`Create a ${tmpShapeType}`">
      <v-card-text>
        <v-text-field v-model="shapeName" label="Name" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="shapeDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            () => {
              tmpShape = [];
              drawMode = true;
              shapeDialog = false;
            }
          "
        >
          Create
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .map-container {
    flex: 2;
  }

  .map {
    height: 824px;
  }

  .layers {
    overflow-y: scroll;
    max-height: 100vh;
  }
</style>
