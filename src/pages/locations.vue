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
  const layersPanelOpen = ref(true);
  const openLayerIds = ref<string[]>([]);
  const editingItemId = ref<string | null>(null);

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

  const tripDialog = ref(false);
  const tripInfo = ref<{
    name?: string;
    shapes?: string[];
    date?: Date;
  }>({});

  function openTripDialog(layer: LayerData['id']) {
    targetLayer.value = layer;
    tripDialog.value = true;
  }

  function toggleLayer(id: string) {
    const idx = openLayerIds.value.indexOf(id);
    if (idx === -1) openLayerIds.value.push(id);
    else openLayerIds.value.splice(idx, 1);
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
  <div class="locations-page">
    <div class="map-fill">
      <div ref="mapEl" class="map" />
    </div>

    <div v-if="fromPhotoLoc" class="photo-loc-bar">
      <span class="photo-loc-text">{{ route.query.center }}</span>
      <v-btn color="primary" density="compact" size="small" @click="openCreateDialog()">
        Create Place Here
      </v-btn>
    </div>

    <div class="layers-float">
      <button class="layers-header" type="button" @click="layersPanelOpen = !layersPanelOpen">
        <v-icon size="16">mdi-layers</v-icon>
        <span class="layers-title">Layers</span>
        <v-icon size="14">{{ layersPanelOpen ? 'mdi-chevron-up' : 'mdi-chevron-down' }}</v-icon>
      </button>
      <v-expand-transition>
        <div v-if="layersPanelOpen" class="layers-body">
          <div class="layers-create-row">
            <v-btn
              color="primary"
              density="compact"
              size="small"
              variant="flat"
              @click="layerDialog = true"
            >
              <v-icon size="14" start>mdi-plus</v-icon>
              Create Layer
            </v-btn>
          </div>
          <div v-for="layer in sortedLayers" :key="layer.id" class="layer-group">
            <div class="layer-row" @click="toggleLayer(layer.id)">
              <span class="layer-dot" :style="{ background: layer.color }" />
              <span class="layer-name">{{ layer.name }}</span>
              <span class="layer-count">{{ layer.count }}</span>
              <v-menu :disabled="drawMode">
                <template #activator="{ props }">
                  <button class="layer-add-btn" type="button" v-bind="props" @click.stop>
                    <v-icon size="14">mdi-plus</v-icon>
                  </button>
                </template>
                <v-list density="compact">
                  <v-list-item @click="openCreateDialog(layer.id)">Add Place</v-list-item>
                  <v-list-item @click="startShape('line', layer.id)">Add Line</v-list-item>
                  <v-list-item @click="startShape('polygon', layer.id)">Add Polygon</v-list-item>
                  <v-list-item @click="openTripDialog(layer.id)">Add Trip</v-list-item>
                </v-list>
              </v-menu>
              <v-icon class="layer-chevron" size="14">
                {{ openLayerIds.includes(layer.id) ? 'mdi-chevron-up' : 'mdi-chevron-down' }}
              </v-icon>
            </div>
            <v-expand-transition>
              <div v-if="openLayerIds.includes(layer.id)" class="layer-items">
                <div
                  v-for="place in placesByLayer(layer.id)"
                  :key="place.id"
                  class="item-row"
                  :class="{ 'item-row--editing': editingItemId === place.id }"
                >
                  <template v-if="editingItemId === place.id">
                    <div class="item-edit-form">
                      <v-text-field
                        v-model="place.name"
                        density="compact"
                        hide-details
                        label="Name"
                        variant="outlined"
                        @update:model-value="
                          async () => {
                            await places[place.id]?.setName(place.name);
                          }
                        "
                      />
                      <v-select
                        v-model="place.category"
                        density="compact"
                        hide-details
                        :items="categories"
                        label="Category"
                        variant="outlined"
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
                      <button class="edit-done-btn" type="button" @click="editingItemId = null">
                        <v-icon size="14">mdi-check</v-icon>
                      </button>
                    </div>
                  </template>
                  <template v-else>
                    <v-icon class="item-icon" size="14">{{ icons[place.category] }}</v-icon>
                    <span class="item-name">{{ place.name }}</span>
                    <v-menu>
                      <template #activator="{ props }">
                        <button class="item-menu-btn" type="button" v-bind="props">
                          <v-icon size="14">mdi-dots-horizontal</v-icon>
                        </button>
                      </template>
                      <v-list density="compact">
                        <v-list-item @click="editingItemId = place.id">Edit</v-list-item>
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
                        <v-list-item @click="deletePlace(place)">Delete</v-list-item>
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
                  </template>
                </div>
                <div
                  v-for="shape in shapesByLayer(layer.id)"
                  :key="shape.id"
                  class="item-row"
                  :class="{ 'item-row--editing': editingItemId === shape.id }"
                >
                  <template v-if="editingItemId === shape.id">
                    <div class="item-edit-form">
                      <v-text-field
                        v-model="shape.name"
                        density="compact"
                        hide-details
                        label="Name"
                        variant="outlined"
                        @update:model-value="
                          async () => {
                            await shapes[shape.id]?.setName(shape.name);
                          }
                        "
                      />
                      <button class="edit-done-btn" type="button" @click="editingItemId = null">
                        <v-icon size="14">mdi-check</v-icon>
                      </button>
                    </div>
                  </template>
                  <template v-else>
                    <v-icon class="item-icon" size="14">
                      {{ shape.type === 'line' ? 'mdi-vector-line' : 'mdi-vector-polygon' }}
                    </v-icon>
                    <span class="item-name">{{ shape.name }}</span>
                    <v-menu>
                      <template #activator="{ props }">
                        <button class="item-menu-btn" type="button" v-bind="props">
                          <v-icon size="14">mdi-dots-horizontal</v-icon>
                        </button>
                      </template>
                      <v-list density="compact">
                        <v-list-item @click="editingItemId = shape.id">Rename</v-list-item>
                        <v-list-item
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
                        </v-list-item>
                        <v-list-item
                          @click="
                            () => {
                              targetShape = shape.id;
                              changeTarget = 'Shape';
                              changeLayerDialog = true;
                            }
                          "
                        >
                          Change Layer
                        </v-list-item>
                        <v-list-item @click="deleteShape(shape.id)">Delete</v-list-item>
                      </v-list>
                    </v-menu>
                  </template>
                </div>
              </div>
            </v-expand-transition>
          </div>
        </div>
      </v-expand-transition>
    </div>

    <div class="map-controls">
      <template v-if="drawMode">
        <v-btn color="primary" size="small" @click="saveShape()">Save Shape</v-btn>
        <v-btn v-if="editingShape" color="primary" size="small" @click="splitShape()">
          Split Shape
        </v-btn>
        <v-btn
          color="error"
          size="small"
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
      </template>
      <v-btn
        size="small"
        @click="
          () => {
            if (hideMarkers) {
              hideMarkers = false;
              map.showAllMarkers();
            } else {
              hideMarkers = true;
              map.hideAllMarkers();
            }
          }
        "
      >
        {{ hideMarkers ? 'Show' : 'Hide' }} Markers
      </v-btn>
      <v-btn
        size="small"
        @click="
          () => {
            if (hideLabels) {
              map.setStyle(Map.DefaultMap);
              hideLabels = false;
              if (hideMarkers) map.hideAllMarkers();
            } else {
              map.setStyle(Map.BlankMap);
              hideLabels = true;
              if (hideMarkers) map.hideAllMarkers();
            }
          }
        "
      >
        {{ hideLabels ? 'Show' : 'Hide' }} Labels
      </v-btn>
    </div>
  </div>

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
  <form-dialog
    v-model="tripDialog"
    :reset="() => (tripInfo = {})"
    title="Add a Trip"
    @submit="async () => {}"
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
      v-model="tripInfo.name"
      color="primary"
      label="Trip Name"
      :rules="[rules.required('A name is required.')]"
    />
    <v-select
      v-model="tripInfo.shapes"
      color="primary"
      label="Shapes"
      :rules="[rules.required('At least one shape is required. Otherwise, what is the point?')]"
    />
    <v-date-input v-model="tripInfo.date" color="primary" label="Trip Date" />
  </form-dialog>
</template>

<style scoped>
  .locations-page {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .map-fill {
    position: absolute;
    inset: 0;
  }

  .map {
    width: 100%;
    height: 100%;
  }

  /* Notice bar for photo-linked navigation (top-center) */
  .photo-loc-bar {
    position: absolute;
    top: var(--space-md);
    left: 50%;
    transform: translateX(-50%);
    z-index: 10;
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 8px 32px oklch(5% 0 0 / 0.6);
    white-space: nowrap;
  }

  .photo-loc-text {
    font-size: var(--text-size-body);
    color: var(--color-text-secondary);
  }

  /* Floating layers card (top-left) */
  .layers-float {
    position: absolute;
    top: var(--space-md);
    left: var(--space-md);
    z-index: 10;
    width: 288px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 8px 32px oklch(5% 0 0 / 0.6);
    overflow: hidden;
  }

  .layers-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    width: 100%;
    padding: 10px var(--space-md);
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-primary);
    font-size: var(--text-size-body);
    font-weight: var(--text-weight-medium);
    letter-spacing: var(--text-tracking-label);
    text-align: left;
    transition: background var(--duration-fast) var(--ease-standard);
  }

  .layers-header:hover {
    background: oklch(94% 0.006 245 / 0.06);
  }

  .layers-title {
    flex: 1;
  }

  .layers-body {
    border-top: 1px solid var(--color-border);
    overflow-y: auto;
    max-height: calc(100vh - 96px);
  }

  .layers-create-row {
    padding: var(--space-sm) var(--space-md);
  }

  /* Map view + draw mode controls (bottom-left) */
  .map-controls {
    position: absolute;
    bottom: var(--space-lg);
    left: var(--space-md);
    z-index: 10;
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
    max-width: calc(100% - 320px);
  }

  /* Dialog-internal map preview */
  .map-container {
    height: 300px;
  }

  /* Layer group rows */
  .layer-group {
    border-bottom: 1px solid oklch(20% 0.004 245);
  }

  .layer-group:last-child {
    border-bottom: none;
  }

  .layer-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px var(--space-md);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-standard);
  }

  .layer-row:hover {
    background: oklch(94% 0.006 245 / 0.05);
  }

  .layer-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .layer-name {
    flex: 1;
    font-size: var(--text-size-body);
    font-weight: var(--text-weight-medium);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .layer-count {
    font-size: var(--text-size-label);
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .layer-add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-secondary);
    opacity: 0;
    flex-shrink: 0;
    transition:
      opacity var(--duration-fast) var(--ease-standard),
      background var(--duration-fast) var(--ease-standard);
  }

  .layer-row:hover .layer-add-btn {
    opacity: 1;
  }

  .layer-add-btn:hover {
    background: oklch(94% 0.006 245 / 0.08);
    color: var(--color-text-primary);
  }

  .layer-chevron {
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  /* Place and shape list rows */
  .layer-items {
    background: oklch(11% 0.003 245);
  }

  .item-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px var(--space-md) 5px 28px;
    min-height: 30px;
    transition: background var(--duration-fast) var(--ease-standard);
  }

  .item-row:hover {
    background: oklch(94% 0.006 245 / 0.04);
  }

  .item-row--editing {
    padding: var(--space-sm) var(--space-md);
    align-items: flex-start;
    flex-direction: column;
  }

  .item-icon {
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .item-name {
    flex: 1;
    font-size: var(--text-size-body);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-menu-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: none;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    color: var(--color-text-secondary);
    opacity: 0;
    flex-shrink: 0;
    transition:
      opacity var(--duration-fast) var(--ease-standard),
      background var(--duration-fast) var(--ease-standard);
  }

  .item-row:hover .item-menu-btn {
    opacity: 1;
  }

  .item-menu-btn:hover {
    background: oklch(94% 0.006 245 / 0.08);
    color: var(--color-text-primary);
  }

  /* Inline edit form (triggered from context menu) */
  .item-edit-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    width: 100%;
  }

  .edit-done-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: flex-end;
    padding: 4px 10px;
    background: oklch(94% 0.006 245 / 0.06);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-primary);
    transition: background var(--duration-fast) var(--ease-standard);
  }

  .edit-done-btn:hover {
    background: oklch(94% 0.006 245 / 0.12);
  }
</style>
