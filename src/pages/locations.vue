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
  setLayerColor,
  createShape,
  setPlaceShape,
  setShapePath,
  setShapeName,
  setPlaceName,
  deletePlace,
  deleteShape,
  setPlaceLayer,
  setShapeLayer,
  setPlaceTags,
  setPlaceNotes,
  setPlaceCategory,
  setPlacePosition,
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
          layers[targetLayer.value]?.data.color,
          placeName.value,
        );
      });
    }, 500);
  }
}

function setShapeColor(shape: Shape, color: string) {
  map.removeShape(shape.Id);
  map.createShape(shape.data.type, shape.points, color, shape.Id, false);
}

function setPlaceColor(place: Place, color: string) {
  map.removeMarker(place.Id);
  map.createMarker(place.pos, place.Id, place.data.category, color, place.data.name, place.count);
  // update color of linked polygons
  const s = shapes[place.data.shape];
  if (place.data.shape.length > 0 && s) {
    setShapeColor(s, color);
  }
}

async function deleteShapeFunc(layer_id: string, id: string) {
  await deleteShape(id);
  map.removeShape(id);
  shapeMap.value[layer_id]?.splice(
    shapeMap.value[layer_id].findIndex(p => p.Id === id),
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
    placeMap.value[layer.Id] = [];
    shapeMap.value[layer.Id] = [];
    layerEntryList.value.push({
      title: layer.data.name,
      value: layer.Id,
    });
  }
  await map.initialize(mapEl.value as unknown as HTMLElement);
  const linkedShapes: string[] = [];
  for (const layer of layerList.value) {
    for (const place of Object.values(places).filter(place => place.data.layer === layer.Id)) {
      if (!placeMap.value[place.data.layer]) {
        placeMap.value[place.data.layer] = [];
      }
      placeMap.value[place.data.layer]?.push(place);
      if (place.data.shape.length > 0) {
        linkedShapes.push(place.data.shape);
      }
      map.createMarker(
        place.pos,
        place.Id,
        place.data.category,
        layers[place.data.layer]?.data.color,
        place.data.name,
        place.count,
      );
    }
    for (const shape of Object.values(shapes).filter(shape => shape.data.layer === layer.Id)) {
      if (!linkedShapes.includes(shape.Id)) {
        if (!shapeMap.value[shape.data.layer]) {
          shapeMap.value[shape.data.layer] = [];
        }
        shapeMap.value[shape.data.layer]?.push(shape);
      }
      totalArea.value += shape.area;
      map.createShape(
        shape.data.type,
        shape.points,
        layers[shape.data.layer]?.data.color ?? '',
        shape.Id,
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
        layers[targetLayer.value]?.data.color ?? '',
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
            <v-expansion-panel v-for="layer in layerList" :key="layer.Id">
              <v-expansion-panel-title>
                {{ layer.data.name }} ({{ placeMap[layer.Id]?.length }})
              </v-expansion-panel-title>
              <v-expansion-panel-text>
                {{ layer.data.name }}
                <v-menu :disabled="drawMode">
                  <template #activator="{ props }">
                    <v-btn flat icon v-bind="props">
                      <v-icon>mdi-plus</v-icon>
                    </v-btn>
                  </template>
                  <v-list>
                    <v-list-item @click="openCreateDialog(layer.Id)">Add Place</v-list-item>
                    <v-list-item
                      @click="
                        () => {
                          tmpShapeType = 'line';
                          targetLayer = layer.Id;
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
                          targetLayer = layer.Id;
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
                  :color="layer.data.color"
                  @update="
                    async color => {
                      // don't know why this line is needed, but it is otherwise shapes in edit mode won't have the right color and new shapes won't either
                      const l = layers[layer.Id];
                      if (l) {
                        l.data.color = color;
                        await setLayerColor(layer.Id, color);
                        // update color of places, lines, and shapes
                        placeMap[layer.Id]?.forEach(place => {
                          setPlaceColor(place, color);
                        });
                        shapeMap[layer.Id]?.forEach(shape => {
                          setShapeColor(shape, color);
                        });
                      }
                    }
                  "
                />
                <v-expansion-panels>
                  <v-expansion-panel v-for="place in placeMap[layer.Id]" :key="place.Id">
                    <v-expansion-panel-title>{{ place.data.name }}</v-expansion-panel-title>
                    <v-expansion-panel-text>
                      <v-btn
                        v-if="place.data.shape.length === 0"
                        color="primary"
                        @click="
                          () => {
                            tmpShape = [];
                            drawMode = true;
                            editingShape = false;
                            tmpShapeType = 'polygon';
                            targetLayer = layer.Id;
                            targetPlace = place.Id;
                          }
                        "
                      >
                        Draw Polygon
                      </v-btn>
                      <div v-if="place.data.shape.length > 0">
                        <v-btn
                          @click="
                            () => {
                              const shape = shapes[place.data.shape];
                              map.removeShape(place.data.shape);
                              targetLayer = layer.Id;
                              if (shape) {
                                tmpShape = shape.points;
                                editingShape = true;
                                targetShape = shape.Id;
                                tmpShapeType = shape.data.type;
                                map.createShape(
                                  shape.data.type,
                                  shape.points,
                                  layers[shape.data.layer]?.data.color ?? '',
                                  shape.Id,
                                  true,
                                );
                                drawMode = true;
                              }
                            }
                          "
                        >
                          Edit Polygon
                        </v-btn>
                        Area: {{ shapes[place.data.shape]?.area }}
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
                                await deletePlace(place.Id);
                                if (place.data.shape.length > 0) {
                                  deleteShapeFunc(layer.Id, place.data.shape);
                                }
                                placeMap[layer.Id]?.splice(
                                  placeMap[layer.Id]?.findIndex(p => p.Id === place.Id) ?? -1,
                                  1,
                                );
                                map.removeMarker(place.Id);
                              }
                            "
                          >
                            Delete Place
                          </v-list-item>
                          <v-list-item
                            @click="
                              () => {
                                targetPlace = place.Id;
                                changeLayerDialog = true;
                              }
                            "
                          >
                            Change Layer
                          </v-list-item>
                          <v-list-item
                            @click="
                              () => {
                                targetLayer = place.data.layer;
                                map.removeMarker(place.Id);
                                editingShape = true;
                                const listener = map.on('dblclick', async pos => {
                                  map.createMarker(
                                    locToString(pos),
                                    place.Id,
                                    place.data.category,
                                    layers[targetLayer]?.data.color,
                                    place.data.name,
                                    place.count,
                                  );
                                  map.off('dblclick', listener);
                                  await setPlacePosition(place.Id, pos);
                                });
                              }
                            "
                          >
                            Move Icon
                          </v-list-item>
                          <v-list-item
                            @click="
                              () => {
                                router.push(`/tagger?place=${place.Id}`);
                              }
                            "
                          >
                            View Photos
                          </v-list-item>
                        </v-list>
                      </v-menu>
                      <v-text-field
                        v-model="place.data.name"
                        label="Name"
                        @update:model-value="
                          async () => {
                            await setPlaceName(place.Id, place.data.name);
                          }
                        "
                      />
                      <v-select
                        v-model="place.data.category"
                        :items="categories"
                        @update:model-value="
                          async () => {
                            await setPlaceCategory(place.Id, place.data.category);
                            map.removeMarker(place.Id);
                            map.createMarker(
                              place.pos,
                              place.Id,
                              place.data.category,
                              layers[place.data.layer]?.data.color,
                              place.data.name,
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
                            await setPlaceTags(place.Id, tags);
                            updateTags(tags);
                          }
                        "
                      />
                      <autosave-text
                        label="Notes"
                        :value="place.data.notes"
                        @save="
                          async notes => {
                            await setPlaceNotes(place.Id, notes);
                          }
                        "
                      />
                    </v-expansion-panel-text>
                  </v-expansion-panel>
                  <v-expansion-panel v-for="shape in shapeMap[layer.Id]" :key="shape.Id">
                    <v-expansion-panel-title>{{ shape.data.name }}</v-expansion-panel-title>
                    <v-expansion-panel-text>
                      <v-text-field
                        v-model="shape.data.name"
                        label="Name"
                        @update:model-value="
                          async () => {
                            await setShapeName(shape.Id, shape.data.name);
                          }
                        "
                      />
                      <v-btn
                        @click="
                          () => {
                            map.removeShape(shape.Id);
                            targetLayer = layer.Id;
                            tmpShape = shape.points;
                            editingShape = true;
                            targetShape = shape.Id;
                            tmpShapeType = shape.data.type;
                            map.createShape(
                              shape.data.type,
                              shape.points,
                              layers[shape.data.layer]?.data.color ?? '',
                              shape.Id,
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
                            targetShape = shape.Id;
                            changeShapeLayerDialog = true;
                          }
                        "
                      >
                        Change Layer
                      </v-btn>
                      <v-btn
                        @click="
                          async () => {
                            deleteShapeFunc(layer.Id, shape.Id);
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
                  await setShapePath(targetShape, tmpShape);
                  map.removeShape(targetShape);
                  map.createShape(
                    shapes[targetShape]?.data.type ?? 'line',
                    tmpShape,
                    layers[targetLayer]?.data.color ?? '',
                    targetShape,
                  );
                } else {
                  const s = await createShape(tmpShapeType, tmpShape, targetLayer, shapeName);
                  if (targetPlace.length > 0) {
                    await setPlaceShape(targetPlace, s);
                  } else {
                    if (!shapeMap[targetLayer]) {
                      shapeMap[targetLayer] = [];
                    }
                    shapeMap[targetLayer]?.push(s);
                  }
                  map.removeShape(`${prevShape}`);
                  map.createShape(
                    tmpShapeType,
                    tmpShape,
                    layers[targetLayer]?.data.color ?? '',
                    s.Id,
                  );
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
                  await setShapePath(targetShape, split1);
                  map.removeShape(targetShape);
                  map.createShape(
                    shapes[targetShape]?.data.type ?? 'line',
                    split1,
                    layers[targetLayer]?.data.color ?? '',
                    targetShape,
                  );
                  const newShape = await createShape(
                    tmpShapeType,
                    split2,
                    targetLayer,
                    `${shapes[targetShape]?.data.name} - Split`,
                  );
                  if (!shapeMap[targetLayer]) {
                    shapeMap[targetLayer] = [];
                  }
                  shapeMap[targetLayer]?.push(newShape);
                  map.createShape(
                    tmpShapeType,
                    split2,
                    layers[targetLayer]?.data.color ?? '',
                    newShape.Id,
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
              placeMap[layer.Id] = [];
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
      <v-card-title>Add a Location to {{ layers[targetLayer]?.data.name }}</v-card-title>
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
              const p = await createPlace(placeName, position, placeCategory, targetLayer);
              placeMap[targetLayer]?.push(p);
              map.createMarker(
                locToString(position),
                p.Id,
                placeCategory,
                layers[targetLayer]?.data.color,
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
      <v-card-title>Change Layer of {{ places[targetPlace]?.data.name }}</v-card-title>
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
              const prevLayer = places[targetPlace]?.data.layer;
              if (prevLayer) {
                placeMap[prevLayer]?.splice(
                  placeMap[prevLayer]?.findIndex(p => p.Id === targetPlace),
                  1,
                );
              }
              await setPlaceLayer(targetPlace, layerChangeTarget);
              const p = places[targetPlace];
              if (p) {
                placeMap[layerChangeTarget]?.push(p);
                const polygon = p.data.shape;
                if (polygon.length > 0) {
                  await setShapeLayer(polygon, layerChangeTarget);
                }
                setPlaceColor(p, layers[layerChangeTarget]?.data.color ?? '');
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
      <v-card-title>Change Layer of {{ shapes[targetShape]?.data.name }}</v-card-title>
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
              const prevLayer = shapes[targetShape]?.data.layer;
              if (prevLayer) {
                shapeMap[prevLayer]?.splice(
                  shapeMap[prevLayer].findIndex(s => s.Id === targetShape),
                  1,
                );
              }
              const s = shapes[targetShape];
              if (s) {
                shapeMap[layerChangeTarget]?.push(s);
                await setShapeLayer(targetShape, layerChangeTarget);
                setShapeColor(s, layers[layerChangeTarget]?.data.color ?? '');
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
