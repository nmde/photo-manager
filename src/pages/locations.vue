<script setup lang="ts">
  import type { Layer } from '../classes/Layer';
  import type { Place } from '../classes/Place';
  import type { Shape, ShapeType } from '../classes/Shape';
  import { computed, onMounted, ref } from 'vue';
  import { useRouter } from 'vue-router';
  import { icons, locToString, Map, type Position } from '../classes/Map';
  import { fileStore } from '../stores/fileStore';

  const router = useRouter();

  const {
    places,
    layers,
    shapes,
    createPlace,
    createLayer,
    createShape,
    deletePlace,
    deleteShape,
    updateTags,
  } = fileStore;

  const layerDialog = ref(false);
  const layerName = ref('');
  const createDialog = ref(false);
  const position = ref<Position>({ lat: 0, lng: 0 });
  const mapEl = ref(null);
  const newPlaceMapEl = ref(null);
  const placeName = ref('');
  const placeCategory = ref<keyof typeof icons>('hospital');
  const mapInitialized = ref(false);
  const layerList = ref<Layer[]>([]);
  const layerEntryList = ref<
    {
      title: string;
      value: string;
    }[]
  >([]);
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
  const layerChangeTarget = ref('');
  const changeShapeLayerDialog = ref(false);
  const totalArea = ref(0);

  const categories = computed(() => Object.keys(icons));

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
            layers[targetLayer.value]?.color,
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
    const s = shapes[place.shape];
    if (place.shape.length > 0 && s) {
      setShapeColor(s, color);
    }
  }

  async function deleteShapeFunc(layer_id: string, id: string) {
    await deleteShape(id);
    map.removeShape(id);
    shapeMap.value[layer_id]?.splice(
      shapeMap.value[layer_id].findIndex(p => p.id === id),
      1,
    );
  }

  let prevShape = 0;
  onMounted(async () => {
    layerList.value = Object.values(layers);
    placeMap.value = {};
    shapeMap.value = {};
    totalArea.value = 0;
    for (const layer of layerList.value) {
      placeMap.value[layer.id] = [];
      shapeMap.value[layer.id] = [];
      layerEntryList.value.push({
        title: layer.name,
        value: layer.id,
      });
    }
    await map.initialize(mapEl.value as unknown as HTMLElement);
    const linkedShapes: string[] = [];
    for (const layer of layerList.value) {
      for (const place of Object.values(places).filter(place => place.layer === layer.id)) {
        if (!placeMap.value[place.layer]) {
          placeMap.value[place.layer] = [];
        }
        placeMap.value[place.layer]?.push(place);
        if (place.shape.length > 0) {
          linkedShapes.push(place.shape);
        }
        map.createMarker(
          place.pos,
          place.id,
          place.category,
          layers[place.layer]?.color,
          place.name,
          place.count,
        );
      }
      for (const shape of Object.values(shapes).filter(shape => shape.layer === layer.id)) {
        if (!linkedShapes.includes(shape.id)) {
          if (!shapeMap.value[shape.layer]) {
            shapeMap.value[shape.layer] = [];
          }
          shapeMap.value[shape.layer]?.push(shape);
        }
        totalArea.value += shape.area;
        map.createShape(
          shape.type,
          shape.points,
          layers[shape.layer]?.color ?? '',
          shape.id,
        );
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
          layers[targetLayer.value]?.color ?? '',
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
  <v-main class="main">
    <v-btn color="primary" @click="layerDialog = true">Add Layer</v-btn>
    <v-container fluid>
      <v-row>
        <v-col cols="4">
          <v-expansion-panels>
            <v-expansion-panel v-for="layer in layerList" :key="layer.id">
              <v-expansion-panel-title>
                {{ layer.name }} ({{ placeMap[layer.id]?.length }})
              </v-expansion-panel-title>
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
                  @update="
                    async color => {
                      // don't know why this line is needed, but it is otherwise shapes in edit mode won't have the right color and new shapes won't either
                      const l = layers[layer.id];
                      await layers[layer.id]?.setColor(color);
                      // update color of places, lines, and shapes
                      placeMap[layer.id]?.forEach(place => {
                        setPlaceColor(place, color);
                      });
                      shapeMap[layer.id]?.forEach(shape => {
                        setShapeColor(shape, color);
                      });
                    }
                  "
                />
                <v-expansion-panels>
                  <v-expansion-panel v-for="place in placeMap[layer.id]" :key="place.id">
                    <v-expansion-panel-title>{{ place.name }}</v-expansion-panel-title>
                    <v-expansion-panel-text>
                      <v-btn
                        v-if="place.shape.length === 0"
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
                      <div v-if="place.shape.length > 0">
                        <v-btn
                          @click="
                            () => {
                              const shape = shapes[place.shape];
                              map.removeShape(place.shape);
                              targetLayer = layer.id;
                              if (shape) {
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
                            }
                          "
                        >
                          Edit Polygon
                        </v-btn>
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
                          <v-list-item
                            @click="
                              async () => {
                                await deletePlace(place.id);
                                if (place.shape.length > 0) {
                                  deleteShapeFunc(layer.id, place.shape);
                                }
                                placeMap[layer.id]?.splice(
                                  placeMap[layer.id]?.findIndex(p => p.id === place.id) ?? -1,
                                  1,
                                );
                                map.removeMarker(place.id);
                              }
                            "
                          >
                            Delete Place
                          </v-list-item>
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
                          <v-list-item
                            @click="
                              () => {
                                targetLayer = place.layer;
                                map.removeMarker(place.id);
                                editingShape = true;
                                const listener = map.on('dblclick', async pos => {
                                  map.createMarker(
                                    locToString(pos),
                                    place.id,
                                    place.category,
                                    layers[targetLayer]?.color,
                                    place.name,
                                    place.count,
                                  );
                                  map.off('dblclick', listener);
                                  await places[place.id]?.setPosition(pos);
                                });
                              }
                            "
                          >
                            Move Icon
                          </v-list-item>
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
                      <tag-input
                        label="Tags"
                        :value="place.tags"
                        @update="
                          async tags => {
                            await places[place.id]?.setTags(tags);
                            updateTags(tags);
                          }
                        "
                      />
                      <autosave-text
                        label="Notes"
                        :value="place.notes"
                        @save="
                          async notes => {
                            await places[place.id]?.setNotes(notes);
                          }
                        "
                      />
                    </v-expansion-panel-text>
                  </v-expansion-panel>
                  <v-expansion-panel v-for="shape in shapeMap[layer.id]" :key="shape.id">
                    <v-expansion-panel-title>{{ shape.name }}</v-expansion-panel-title>
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
                            deleteShapeFunc(layer.id, shape.id);
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
          <v-btn
            v-if="drawMode"
            color="primary"
            @click="
              async () => {
                if (editingShape) {
                  await shapes[targetShape]?.setPoints(tmpShape);
                  map.removeShape(targetShape);
                  map.createShape(
                    shapes[targetShape]?.type ?? 'line',
                    tmpShape,
                    layers[targetLayer]?.color ?? '',
                    targetShape,
                  );
                } else {
                  const s = await createShape(tmpShapeType, tmpShape, targetLayer, shapeName);
                  if (targetPlace.length > 0) {
                    await places[targetPlace]?.setShape(s.id);
                  } else {
                    if (!shapeMap[targetLayer]) {
                      shapeMap[targetLayer] = [];
                    }
                    shapeMap[targetLayer]?.push(s);
                  }
                  map.removeShape(`${prevShape}`);
                  map.createShape(tmpShapeType, tmpShape, layers[targetLayer]?.color ?? '', s.id);
                }
                prevShape = 0;
                drawMode = false;
              }
            "
          >
            Save Shape
          </v-btn>
          <v-btn
            v-if="drawMode"
            color="primary"
            @click="
              async () => {
                if (lastChangedIndex > 0 && lastChangedIndex < tmpShape.length - 1) {
                  const split1 = tmpShape.slice(0, lastChangedIndex);
                  const split2 = tmpShape.slice(lastChangedIndex + 1, tmpShape.length);
                  await shapes[targetShape]?.setPoints(split1);
                  map.removeShape(targetShape);
                  map.createShape(
                    shapes[targetShape]?.type ?? 'line',
                    split1,
                    layers[targetLayer]?.color ?? '',
                    targetShape,
                  );
                  const newShape = await createShape(
                    tmpShapeType,
                    split2,
                    targetLayer,
                    `${shapes[targetShape]?.name} - Split`,
                  );
                  if (!shapeMap[targetLayer]) {
                    shapeMap[targetLayer] = [];
                  }
                  shapeMap[targetLayer]?.push(newShape);
                  map.createShape(
                    tmpShapeType,
                    split2,
                    layers[targetLayer]?.color ?? '',
                    newShape.id,
                  );
                  drawMode = false;
                }
              }
            "
          >
            Splt Shape
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
  </v-main>
  <v-dialog v-model="layerDialog">
    <v-card>
      <v-card-title>Create a Layer</v-card-title>
      <v-card-text>
        <v-text-field v-model="layerName" label="Name" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="layerDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              const layer = await createLayer(layerName);
              layerList.push(layer);
              placeMap[layer.id] = [];
              layerDialog = false;
              layerName = '';
            }
          "
        >
          Create Layer
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="createDialog">
    <v-card>
      <v-card-title>Add a Location to {{ layers[targetLayer]?.name }}</v-card-title>
      <v-card-text>
        <v-text-field v-model="placeName" label="Name" />
        Category: <v-select v-model="placeCategory" :items="categories" />
        <div class="map-container">
          <div ref="newPlaceMapEl" class="map" />
        </div>
        Selected position: {{ position }}<br>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              const p = await createPlace(placeName, position, placeCategory, targetLayer);
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
    <v-card>
      <v-card-title>Create a {{ tmpShapeType }}</v-card-title>
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
  <v-dialog v-model="changeLayerDialog">
    <v-card>
      <v-card-title>Change Layer of {{ places[targetPlace]?.name }}</v-card-title>
      <v-card-text>
        <v-select v-model="layerChangeTarget" :items="layerEntryList" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="changeLayerDialog = false">Cancel</v-btn>
        <v-btn
          v-if="layerChangeTarget.length > 0"
          color="primary"
          @click="
            async () => {
              const prevLayer = places[targetPlace]?.layer;
              if (prevLayer) {
                placeMap[prevLayer]?.splice(
                  placeMap[prevLayer]?.findIndex(p => p.id === targetPlace),
                  1,
                );
              }
              const p = places[targetPlace];
              if (p) {
                p.setLayer(layerChangeTarget);
                placeMap[layerChangeTarget]?.push(p);
                const polygon = p.shape;
                if (polygon.length > 0) {
                  await shapes[polygon]?.setLayer(layerChangeTarget);
                }
                setPlaceColor(p, layers[layerChangeTarget]?.color ?? '');
              }
              changeLayerDialog = false;
              layerChangeTarget = '';
            }
          "
        >
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="changeShapeLayerDialog">
    <v-card>
      <v-card-title>Change Layer of {{ shapes[targetShape]?.name }}</v-card-title>
      <v-card-text>
        <v-select v-model="layerChangeTarget" :items="layerEntryList" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="changeShapeLayerDialog = false">Cancel</v-btn>
        <v-btn
          v-if="layerChangeTarget.length > 0"
          color="primary"
          @click="
            async () => {
              const prevLayer = shapes[targetShape]?.layer;
              if (prevLayer) {
                shapeMap[prevLayer]?.splice(
                  shapeMap[prevLayer].findIndex(s => s.id === targetShape),
                  1,
                );
              }
              const s = shapes[targetShape];
              if (s) {
                shapeMap[layerChangeTarget]?.push(s);
                await shapes[targetShape]?.setLayer(layerChangeTarget);
                setShapeColor(s, layers[layerChangeTarget]?.color ?? '');
              }
              changeShapeLayerDialog = false;
              layerChangeTarget = '';
            }
          "
        >
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .main {
    margin: 8px;
  }

  .map-container {
    flex: 2;
  }

  .map {
    height: 824px;
  }

  .layer {
    width: 100%;
  }
</style>
