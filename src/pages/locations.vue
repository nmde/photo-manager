<script setup lang="ts">
  import { v4 as uuid } from 'uuid';
  import { useRules } from 'vuetify/labs/rules';
  import {
    create_layer,
    create_place,
    create_shape,
    delete_place,
    delete_shape,
    get_layers,
    get_places,
    get_shapes,
    set_place_layer,
    set_shape_layer,
  } from '@/api/places';
  import { Layer, type LayerData, type LayerRec } from '@/classes/Layer';
  import { icons, locToString, Map, type PlaceType, type Position } from '@/classes/Map';
  import { Place, type PlaceRec } from '@/classes/Place';
  import { Shape, type ShapeRec, type ShapeType } from '@/classes/Shape';
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const { reportError } = store;
  const router = useRouter();
  const route = useRoute();
  const rules = useRules();

  const createDialog = ref(false);
  const mapEl = useTemplateRef('mapEl');
  const newPlaceMapEl = useTemplateRef('newPlaceMapEl');
  const mapInitialized = ref(false);
  const layers = ref<LayerRec>({});
  const targetLayer = ref<string>();
  const drawMode = ref(false);
  const tmpShape = ref<Position[]>([]);
  const tmpShapeType = ref<ShapeType>('line');
  const shapeName = ref('');
  const shapeDialog = ref(false);
  const targetPlace = ref('');
  const editingShape = ref(false);
  const targetShape = ref('');
  const hideMarkers = ref(false);
  const hideLabels = ref(false);
  const changeLayerDialog = ref(false);
  const changeTarget = ref<'Place' | 'Shape'>('Place');
  const places = ref<PlaceRec>({});
  const shapes = ref<ShapeRec>({});
  const fromPhotoLoc = ref(false);
  const queryLoc = ref<Position>({ lat: 0, lng: 0 });
  const layerDialog = ref(false);
  const missingLayerColor = ref(false);
  const missingNewPlaceLoc = ref(false);

  type PlaceFields = {
    name?: string;
    category?: PlaceType;
    position?: { lat: number; lng: number };
  };
  const placeFields = ref<PlaceFields>({});

  type LayerFields = { name?: string; color?: string };
  const layerFields = ref<LayerFields>({});

  const categories = computed(() => Object.keys(icons));

  const sortedLayers = computed(() =>
    Object.values(layers.value).toSorted((a, b) => a.name.localeCompare(b.name)),
  );

  const map = new Map();
  const newPlaceMap = new Map();
  let lastChangedIndex = -1;

  function openCreateDialog(layer?: string) {
    targetLayer.value = layer ?? '';
    createDialog.value = true;
    if (mapInitialized.value) {
      newPlaceMap.clearMarkers();
    } else {
      setTimeout(async () => {
        await newPlaceMap.initialize(newPlaceMapEl.value as unknown as HTMLElement);
        const { center, zoom } = map.getCenter();
        newPlaceMap.setCenter(center?.lat() ?? 0, center?.lng() ?? 0, zoom);

        let existingMarker: string | undefined = undefined;
        if (fromPhotoLoc.value) {
          existingMarker = newPlaceMap.createMarker(
            locToString(queryLoc.value),
            '0',
            placeFields.value.category,
            layers.value[targetLayer.value ?? '']?.color,
            placeFields.value.name,
          );
          placeFields.value.position = queryLoc.value;
        }

        newPlaceMap.on('dblclick', pos => {
          if (existingMarker !== undefined) {
            newPlaceMap.removeMarker(existingMarker);
          }
          placeFields.value.position = pos;
          existingMarker = newPlaceMap.createMarker(
            locToString(pos),
            '0',
            placeFields.value.category,
            layers.value[targetLayer.value ?? '']?.color,
            placeFields.value.name,
          );
        });
      }, 500);
    }
  }

  function setShapeColor(shape: Shape, color: string) {
    map.removeShape(shape.id);
    map.createShape(shape.type, shape.shape, color, shape.id, false);
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
    for (const place of placesByLayer(layer.id)) {
      setPlaceColor(place, color);
    }
    for (const shape of shapesByLayer(layer.id)) {
      setShapeColor(shape, color);
    }
  }

  async function deleteShape(id: string) {
    await delete_shape(id);
    map.removeShape(id);
    delete shapes.value[id];
  }

  async function createShape(type: ShapeType, points: Position[], layer: string, name: string) {
    const id = uuid();
    await create_shape(id, type, JSON.stringify(points), layer, name);
    const shape = new Shape(id, type, JSON.stringify(points), layer, name);
    shapes.value[id] = shape;
    return shape;
  }

  async function createPlace() {
    const id = uuid();
    const fields = placeFields.value as Required<PlaceFields>;
    const layer = targetLayer.value;
    if (layer !== undefined) {
      await create_place(
        id,
        fields.name,
        fields.position.lat,
        fields.position.lng,
        layer,
        fields.category,
      );
      const place = new Place(
        id,
        fields.name,
        fields.position.lat,
        fields.position.lng,
        layer,
        fields.category,
        null,
        0,
      );
      places.value[id] = place;
      map.createMarker(
        locToString(fields.position),
        id,
        fields.category,
        layers.value[layer]?.color,
        fields.name,
      );
      placeFields.value = {};
    }
  }

  async function deletePlace(place: Place) {
    await delete_place(place.id);
    if (place.shape !== null) {
      await deleteShape(place.shape);
    }
    delete places.value[place.id];
    map.removeMarker(place.id);
  }

  function editPlaceShape(place: Place) {
    if (place.shape !== null) {
      const shape = shapes.value[place.shape];
      map.removeShape(place.shape);
      targetLayer.value = place.layer;
      if (shape) {
        tmpShape.value = shape.shape;
        editingShape.value = true;
        targetShape.value = shape.id;
        tmpShapeType.value = shape.type;
        map.createShape(
          shape.type,
          shape.shape,
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
    if (targetLayer.value !== undefined) {
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
          shapes.value[s.id] = s;
        }
        map.removeShape(`${prevShape}`);
        map.createShape(
          tmpShapeType.value,
          tmpShape.value,
          layers.value[targetLayer.value]?.color ?? '',
          s.id,
        );
      }
    }
    prevShape = 0;
    drawMode.value = false;
  }

  async function splitShape() {
    if (
      lastChangedIndex > 0
      && lastChangedIndex < tmpShape.value.length - 1
      && targetLayer.value !== undefined
    ) {
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
      shapes.value[newShape.id] = newShape;
      map.createShape(
        tmpShapeType.value,
        split2,
        layers.value[targetLayer.value]?.color ?? '',
        newShape.id,
      );
      drawMode.value = false;
    }
  }

  async function createLayer() {
    const fields = layerFields.value as Required<LayerFields>;
    const id = uuid();
    await create_layer(id, fields.name, fields.color);
    layers.value[id] = new Layer(id, fields.name, fields.color, 0);
    layerFields.value = {};
  }

  function placesByLayer(layer: LayerData['id']) {
    return Object.values(places.value)
      .filter(p => p.layer === layer)
      .toSorted((a, b) => a.name.localeCompare(b.name));
  }

  function shapesByLayer(layer: LayerData['id']) {
    return Object.values(shapes.value)
      .filter(p => p.layer === layer)
      .toSorted((a, b) => a.name.localeCompare(b.name));
  }

  function startShape(type: ShapeType, layer: LayerData['id']) {
    tmpShapeType.value = type;
    targetLayer.value = layer;
    targetPlace.value = '';
    shapeName.value = '';
    shapeDialog.value = true;
    editingShape.value = false;
  }

  let prevShape = 0;
  onMounted(async () => {
    await get_layers()
      .ok(l => (layers.value = l))
      .err(reportError)
      .send();
    await get_places()
      .ok(p => (places.value = p))
      .err(reportError)
      .send();
    await get_shapes()
      .ok(s => (shapes.value = s))
      .err(reportError)
      .send();
    if (mapEl.value) {
      await map.initialize(mapEl.value);
    }
    const linkedShapes: string[] = [];
    for (const id in layers.value) {
      for (const place of Object.values(places.value).filter(place => place.layer === id)) {
        places.value[place.id] = place;
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
          shapes.value[shape.id] = shape;
        }
        map.createShape(shape.type, shape.shape, layers.value[shape.layer]?.color ?? '', shape.id);
      }
    }

    if (typeof route.query.center === 'string') {
      const split = route.query.center.split(',').map(Number);
      map.setCenter(split[0] ?? 0, split[1] ?? 0);
      fromPhotoLoc.value = true;
      queryLoc.value = {
        lat: split[0] ?? 0,
        lng: split[1] ?? 0,
      };
      map.createMarker(locToString(queryLoc.value), 'focus');
    } else {
      navigator.geolocation.getCurrentPosition(position => {
        map.setCenter(position.coords.latitude, position.coords.longitude);
      });
    }

    map.on('click', pos => {
      if (drawMode.value && targetLayer.value !== undefined) {
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
  <v-container class="fill-height" fluid>
    <v-row class="fill-height" no-gutters>
      <v-col class="layers" cols="3">
        <v-btn color="primary" @click="layerDialog = true">Create Layer</v-btn>
        <v-expansion-panels>
          <v-expansion-panel v-for="layer in sortedLayers" :key="layer.id" :color="layer.color">
            <template #title>
              {{ layer.name }} ({{ layer.count }})
              <v-menu :disabled="drawMode">
                <template #activator="{ props }">
                  <v-btn icon v-bind="props" variant="plain">
                    <v-icon>mdi-plus</v-icon>
                  </v-btn>
                </template>
                <v-list>
                  <v-list-item @click="openCreateDialog(layer.id)">Add Place</v-list-item>
                  <v-list-item @click="startShape('line', layer.id)">Add Line</v-list-item>
                  <v-list-item @click="startShape('polygon', layer.id)">Add Polygon</v-list-item>
                </v-list>
              </v-menu>
            </template>
            <v-expansion-panel-text>
              <v-expansion-panels>
                <v-expansion-panel v-for="place in placesByLayer(layer.id)" :key="place.id">
                  <v-expansion-panel-title>
                    <v-icon>{{ icons[place.category] }}</v-icon>
                    &nbsp;{{ place.name }}
                  </v-expansion-panel-title>
                  <v-expansion-panel-text>
                    <v-menu>
                      <template #activator="{ props }">
                        <v-btn flat icon v-bind="props">
                          <v-icon>mdi-menu</v-icon>
                        </v-btn>
                      </template>
                      <v-list>
                        <v-list-item
                          v-if="place.shape === null"
                          @click="
                            () => {
                              tmpShape = [];
                              drawMode = true;
                              editingShape = false;
                              shapeName = `${place.name} Polygon`;
                              tmpShapeType = 'polygon';
                              targetLayer = layer.id;
                              targetPlace = place.id;
                            }
                          "
                        >
                          Draw Polygon
                        </v-list-item>
                        <v-list-item v-else @click="editPlaceShape(place)">
                          Edit Polygon
                        </v-list-item>
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
                              changeTarget = 'Place';
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
                <v-expansion-panel v-for="shape in shapesByLayer(layer.id)" :key="shape.id">
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
                          tmpShape = shape.shape;
                          editingShape = true;
                          targetShape = shape.id;
                          tmpShapeType = shape.type;
                          map.createShape(
                            shape.type,
                            shape.shape,
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
                          changeTarget = 'Shape';
                          changeLayerDialog = true;
                        }
                      "
                    >
                      Change Layer
                    </v-btn>
                    <v-btn @click="deleteShape(shape.id)">Delete Shape</v-btn>
                  </v-expansion-panel-text>
                </v-expansion-panel>
              </v-expansion-panels>
            </v-expansion-panel-text>
          </v-expansion-panel>
        </v-expansion-panels>
      </v-col>
      <v-col cols="9">
        <v-alert v-if="fromPhotoLoc" type="info">
          Viewing {{ route.query.center }}
          <v-btn color="primary" density="comfortable" @click="openCreateDialog()">
            Create Place Here
          </v-btn>
        </v-alert>
        <div class="map-container">
          <div ref="mapEl" class="map" />
        </div>
        <v-btn v-if="drawMode" color="primary" @click="saveShape()">Save Shape</v-btn>
        <v-btn v-if="drawMode && editingShape" color="primary" @click="splitShape()">
          Splt Shape
        </v-btn>
        <v-btn
          v-if="drawMode"
          color="error"
          @click="
            () => {
              tmpShape = [];
              const x = shapes[targetShape];
              if (editingShape && x !== undefined) {
                map.createShape(x.type, x.shape, layers[x.layer]?.color ?? '', targetShape, false);
              }
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
  <form-dialog
    v-model="createDialog"
    :reset="
      () => {
        targetLayer = undefined;
        placeFields = {};
      }
    "
    :title="`Add a Location to ${targetLayer ? (layers[targetLayer]?.name ?? '...') : '...'}`"
    @submit="
      async () => {
        missingNewPlaceLoc = placeFields.position === undefined;
        if (!missingNewPlaceLoc) {
          await createPlace();
        }
      }
    "
  >
    <v-select
      v-model="targetLayer"
      color="primary"
      item-title="name"
      item-value="id"
      :items="Object.values(layers)"
      label="Layer"
      :rules="[rules.required('A layer is required.')]"
    />
    <v-text-field
      v-model="placeFields.name"
      color="primary"
      label="Name"
      :rules="[rules.required('A place name is required.')]"
    />
    <v-select
      v-model="placeFields.category"
      color="primary"
      :items="categories"
      label="Category"
      :rules="[rules.required('A category is required.')]"
    />
    <br />
    <div class="map-container">
      <div ref="newPlaceMapEl" class="map" />
    </div>
    <error-hint :message="missingNewPlaceLoc ? 'A location is required.' : undefined" />
    <br />
    Selected position: {{ placeFields.position }}<br />
  </form-dialog>
  <form-dialog
    v-model="shapeDialog"
    :reset="() => (shapeName = '')"
    :title="`Create a ${tmpShapeType}`"
    @submit="
      () => {
        tmpShape = [];
        drawMode = true;
        shapeDialog = false;
      }
    "
  >
    <v-text-field
      v-model="shapeName"
      label="Name"
      :rules="[rules.required('A shape name is required.')]"
    />
  </form-dialog>
  <form-dialog
    v-model="layerDialog"
    :reset="() => (layerFields = {})"
    title="Create Layer"
    @submit="
      async () => {
        missingLayerColor = layerFields.color === undefined;
        if (!missingLayerColor) {
          await createLayer();
        }
      }
    "
  >
    <v-text-field
      v-model="layerFields.name"
      color="primary"
      label="Name"
      :rules="[rules.required('Layer name is required')]"
    />
    <color-options
      :error="missingLayerColor"
      :value="layerFields.color"
      @select="color => (layerFields.color = color ?? '')"
    />
  </form-dialog>
  <form-dialog
    v-model="changeLayerDialog"
    :reset="() => (targetLayer = undefined)"
    title="Change Layer"
    @submit="
      async () => {
        if (changeTarget === 'Place') {
          await set_place_layer(targetPlace, targetLayer as string);
        } else {
          await set_shape_layer(targetShape, targetLayer as string);
        }
      }
    "
  >
    <v-select
      v-model="targetLayer"
      color="primary"
      item-title="name"
      item-value="id"
      :items="Object.values(layers)"
      label="Layer"
      :rules="[rules.required('A layer is required.')]"
    />
  </form-dialog>
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
