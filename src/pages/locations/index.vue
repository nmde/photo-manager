<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { Map, type Position, locToString, icons } from '../../classes/Map';
import { fileStore } from '../../stores/fileStore';
import type { Layer } from '~/classes/Layer';
import type { Shape, ShapeType } from '~/classes/Shape';
import type { Place } from '~/classes/Place';

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
const targetLayer = ref('');
const drawMode = ref(false);
const tmpShape = ref<Position[]>([]);
const tmpShapeType = ref<ShapeType>('line');
const placeMap = ref<Record<string, Place[]>>({});
const shapeMap = ref<Record<string, Shape[]>>({});
const shapeName = ref('');
const shapeDialog = ref(false);
const targetPlace = ref('');

const categories = computed(() => Object.keys(icons));

const map = new Map();
const newPlaceMap = new Map();

async function openCreateDialog(layer: string) {
  targetLayer.value = layer;
  createDialog.value = true;
  if (!mapInitialized.value) {
    setTimeout(async () => {
      await newPlaceMap.initialize(newPlaceMapEl.value as unknown as HTMLElement);
      newPlaceMap.on('dblclick', async (pos) => {
        position.value = pos;
        newPlaceMap.createMarker(
          locToString(pos),
          placeCategory.value,
          layers[targetLayer.value].data.color,
          placeName.value,
        );
      });
    }, 500);
  } else {
    newPlaceMap.clearMarkers();
  }
}

let prevShape = '';
onMounted(async () => {
  layerList.value = Object.values(layers);
  const linkedShapes: string[] = [];
  placeMap.value = {};
  shapeMap.value = {};
  Object.values(places).forEach((place) => {
    if (!placeMap.value[place.data.layer]) {
      placeMap.value[place.data.layer] = [];
    }
    placeMap.value[place.data.layer].push(place);
    if (place.data.shape.length > 0) {
      linkedShapes.push(place.data.shape);
    }
  });
  Object.values(shapes)
    .filter((shape) => linkedShapes.indexOf(shape.Id) < 0)
    .forEach((shape) => {
      if (!shapeMap.value[shape.data.layer]) {
        shapeMap.value[shape.data.layer] = [];
      }
      shapeMap.value[shape.data.layer].push(shape);
    });
  await map.initialize(mapEl.value as unknown as HTMLElement);
  Object.values(places).forEach((place) => {
    map.createMarker(
      place.pos,
      place.data.category,
      layers[place.data.layer].data.color,
      place.data.name,
      place.Id,
    );
  });
  Object.values(shapes).forEach((shape) => {
    map.createShape(shape.data.type, shape.points, layers[shape.data.layer].data.color);
  });
  map.on('click', (pos) => {
    if (drawMode.value) {
      tmpShape.value.push(pos);
      if (prevShape.length > 0) {
        map.removeShape(prevShape);
      }
      prevShape = map.createShape(
        tmpShapeType.value,
        tmpShape.value,
        layers[targetLayer.value].data.color,
        true,
      );
    }
  });
  map.on('shapeUpdate', (pos) => {
    if (drawMode.value) {
      pos.forEach((item, i) => {
        tmpShape.value[i] = item.toJSON();
      });
    }
  });
  map.on('markerClicked', (place) => {
    router.push(`/tagger?place=${place}`);
  });
});
</script>

<template>
  <v-main class="main">
    <v-btn color="primary" @click="layerDialog = true">Add Layer</v-btn>
    <v-container fluid>
      <v-row>
        <v-col cols="4">
          <v-card class="layer" v-for="layer in layerList" :key="layer.Id">
            <v-card-title
              >{{ layer.data.name }}
              <v-menu :disabled="drawMode">
                <template v-slot:activator="{ props }">
                  <v-btn icon flat v-bind="props">
                    <v-icon>mdi-plus</v-icon>
                  </v-btn>
                </template>
                <v-list>
                  <v-list-item @click="openCreateDialog(layer.Id)">Add Place</v-list-item>
                  <v-list-item
                    @click="
                      () => {
                        targetLayer = layer.Id;
                        targetPlace = '';
                        shapeName = '';
                        shapeDialog = true;
                      }
                    "
                    >Add Shape</v-list-item
                  >
                </v-list>
              </v-menu>
              <color-picker
                :color="layer.data.color"
                @update="
                  async (color) => {
                    await setLayerColor(layer.Id, color);
                  }
                "
              ></color-picker>
            </v-card-title>
            <v-card-text>
              <v-expansion-panels>
                <v-expansion-panel v-for="place in placeMap[layer.Id]" :key="place.Id">
                  <v-expansion-panel-title>{{ place.data.name }}</v-expansion-panel-title>
                  <v-expansion-panel-text>
                    <v-btn
                      color="primary"
                      @click="
                        () => {
                          tmpShape = [];
                          drawMode = true;
                          tmpShapeType = 'polygon';
                          targetLayer = layer.Id;
                          targetPlace = place.Id;
                        }
                      "
                      >Draw Polygon</v-btn
                    >
                    <v-text-field label="Name" v-model="place.data.name"></v-text-field>
                    <v-select
                      :items="categories"
                      v-model="place.data.category"
                      @update:model-value="() => {}"
                    ></v-select>
                    <v-textarea label="Notes"></v-textarea>
                  </v-expansion-panel-text>
                </v-expansion-panel>
                <v-expansion-panel v-for="shape in shapeMap[layer.Id]" :key="shape.Id">
                  <v-expansion-panel-title>{{ shape.data.name }}</v-expansion-panel-title>
                  <v-expansion-panel-text></v-expansion-panel-text>
                </v-expansion-panel>
              </v-expansion-panels>
            </v-card-text>
          </v-card>
        </v-col>
        <v-col cols="8">
          <div class="map-container">
            <div ref="mapEl" class="map"></div>
          </div>
          <v-btn
            v-if="drawMode"
            color="primary"
            @click="
              async () => {
                const s = await createShape(tmpShapeType, tmpShape, targetLayer, shapeName);
                if (!shapeMap[targetLayer]) {
                  shapeMap[targetLayer] = [];
                }
                shapeMap[targetLayer].push(s);
                if (targetPlace.length > 0) {
                  await setPlaceShape(targetPlace, s);
                }
                drawMode = false;
              }
            "
            >Save Shape</v-btn
          >
        </v-col>
      </v-row>
    </v-container>
  </v-main>
  <v-dialog v-model="layerDialog">
    <v-card>
      <v-card-title>Create a Layer</v-card-title>
      <v-card-text>
        <v-text-field label="Name" v-model="layerName"></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="layerDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              layerList.push(await createLayer(layerName));
              layerDialog = false;
              layerName = '';
            }
          "
          >Create Layer</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="createDialog">
    <v-card>
      <v-card-title>Add a Location to {{ layers[targetLayer].data.name }}</v-card-title>
      <v-card-text>
        <v-text-field label="Name" v-model="placeName"></v-text-field>
        Category: <v-select :items="categories" v-model="placeCategory"></v-select>
        <div class="map-container">
          <div ref="newPlaceMapEl" class="map"></div>
        </div>
        Selected position: {{ position }}<br />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              placeMap[targetLayer].push(
                await createPlace(placeName, position, placeCategory, targetLayer),
              );
              map.createMarker(
                locToString(position),
                placeCategory,
                layers[targetLayer].data.color,
                placeName,
              );
              createDialog = false;
              placeName = '';
              placeCategory = 'hospital';
              position = { lat: 0, lng: 0 };
            }
          "
          >Save</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="shapeDialog">
    <v-card>
      <v-card-title>Create a Shape</v-card-title>
      <v-card-text>
        <v-text-field label="Name" v-model="shapeName"></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="shapeDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            () => {
              tmpShape = [];
              drawMode = true;
              tmpShapeType = 'line';
              shapeDialog = false;
            }
          "
          >Create</v-btn
        >
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
