<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { Map, type Position, locToString } from '../../classes/Map';
import { fileStore } from '../../stores/fileStore';
import type { Layer } from '~/classes/Layer';

const { places, layers, createPlace, createLayer, getPlacesByLayer, setLayerColor } = fileStore;

const layerDialog = ref(false);
const layerName = ref('');
const createDialog = ref(false);
const position = ref<Position>({ lat: 0, lng: 0 });
const mapEl = ref(null);
const newPlaceMapEl = ref(null);
const placeName = ref('');
const mapInitialized = ref(false);
const layerList = ref<Layer[]>([]);
const targetLayer = ref('');

const map = new Map();
const newPlaceMap = new Map();

async function openCreateDialog(layer: string) {
  targetLayer.value = layer;
  createDialog.value = true;
  if (!mapInitialized.value) {
    setTimeout(async () => {
      await newPlaceMap.initialize(newPlaceMapEl.value as unknown as HTMLElement);
      newPlaceMap.on('markerCreated', async (pos) => {
        position.value = pos;
      });
    }, 500);
  } else {
    newPlaceMap.clearMarkers();
  }
}

onMounted(async () => {
  layerList.value = Object.values(layers);
  await map.initialize(mapEl.value as unknown as HTMLElement);
  Object.values(places).forEach((place) => {
    map.createMarker(place.pos, 'hospital', layers[place.data.layer].data.color, place.data.name);
  });
});
</script>

<template>
  <v-main class="main">
    <v-btn color="primary" @click="layerDialog = true">Add Layer</v-btn>
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <v-card class="layer" v-for="layer in layerList" :key="layer.Id">
            <v-card-title
              >{{ layer.data.name }}
              <v-btn icon flat @click="openCreateDialog(layer.Id)">
                <v-icon>mdi-plus</v-icon>
              </v-btn>
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
                <v-expansion-panel v-for="place in getPlacesByLayer(layer.Id)" :key="place.Id">
                  <v-expansion-panel-title>{{ place.data.name }}</v-expansion-panel-title>
                  <v-expansion-panel-text>
                    <v-text-field label="Name" v-model="place.data.name"></v-text-field>
                    <v-textarea label="Notes"></v-textarea>
                  </v-expansion-panel-text>
                </v-expansion-panel>
              </v-expansion-panels>
            </v-card-text>
          </v-card>
        </v-col>
        <v-col cols="6">
          <div class="map-container">
            <div ref="mapEl" class="map"></div>
          </div>
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
              await createPlace(placeName, position, targetLayer);
              map.createMarker(
                locToString(position),
                'hospital',
                layers[targetLayer].data.color,
                placeName,
              );
              createDialog = false;
              placeName = '';
              position = { lat: 0, lng: 0 };
            }
          "
          >Save</v-btn
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
  height: 450px;
}

.layer {
  width: 100%;
}
</style>
