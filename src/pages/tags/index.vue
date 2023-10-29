<script setup lang="ts">
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale, Tooltip } from 'chart.js';
import { Bar } from 'vue-chartjs';
import { useFileStore } from '../../stores/fileStore';
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Tag } from '../../classes/Tag';

/**
 * TODO:
 * - Delete a tag
 * - View redundant tags
 * - Tag network
 * - Tag influence on rating
 */

ChartJS.register(CategoryScale, LinearScale, BarElement, Tooltip);

const fileStore = useFileStore();
const { setTagColor, setTagPrereqs, setTagIncompatible, getTagColor } = fileStore;
const { tagCounts, advTags } = storeToRefs(fileStore);

const cutoff = ref(1);
const selected = ref('');
const selectedColor = ref('black');
const prereqTags = ref<string[]>([]);
const incompatibleTags = ref<string[]>([]);

const colors = [
  '#F44336',
  '#E91E63',
  '#9C27B0',
  '#673AB7',
  '#3F51B5',
  '#2196F3',
  '#03A9F4',
  '#00BCD4',
  '#009688',
  '#4CAF50',
  '#8BC34A',
  '#CDDC39',
  '#FFEB3B',
  '#FFC107',
  '#FF9800',
  '#FF5722',
];

const tagChartData = computed(() => {
  let sorted: string[] = [];
  const backgroundColor: string[] = [];
  Object.entries(tagCounts.value)
    .filter((count) => count[1] >= cutoff.value)
    .forEach(([tag, value]) => {
      let color = getTagColor(tag);
      if (color === 'black') {
        color = 'rgba(201, 203, 207, 0.8)';
      }
      if (sorted.length === 0) {
        sorted.push(tag);
        backgroundColor.push(color);
      } else {
        let i = 0;
        while (i < sorted.length && value < tagCounts.value[sorted[i]]) {
          i += 1;
        }
        sorted.splice(i, 0, tag);
        backgroundColor.splice(i, 0, color);
      }
    });
  return {
    labels: sorted,
    datasets: [
      {
        axis: 'y',
        label: 'Count',
        data: sorted.map((tag) => tagCounts.value[tag]),
        backgroundColor,
      },
    ],
  };
});
</script>

<template>
  <v-main>
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <tag-input
            label="Select a tag"
            :value="selected"
            single
            @update="
              (tags) => {
                selected = tags;
                const adv = (advTags as Tag[]).find((t) => t.data.name === selected);
                selectedColor = getTagColor(selected);
                if (adv) {
                  prereqTags = adv.prereqs;
                  incompatibleTags = adv.incompatible;
                } else {
                  prereqTags = [];
                  incompatibleTags = [];
                }
              }
            "
          ></tag-input>
          <div v-if="selected.length > 0">
            Editing properties of <span :style="{ color: selectedColor }">{{ selected }}</span>
            <br />
            Set color:
            <div class="color-opts">
              <div
                v-for="color in colors"
                :key="color"
                class="color-opt"
                :style="{ 'background-color': color }"
                @click="
                  async () => {
                    selectedColor = color;
                    await setTagColor(selected, color);
                  }
                "
              ></div>
            </div>
            <br />
            <tag-input
              label="Prerequisite Tags"
              :value="prereqTags"
              @update="
                async (tags) => {
                  await setTagPrereqs(selected, tags);
                }
              "
            ></tag-input>
            <tag-input
              label="Incompatible Tags"
              :value="incompatibleTags"
              @update="
                async (tags) => {
                  await setTagIncompatible(selected, tags);
                }
              "
            ></tag-input>
          </div>
        </v-col>
        <v-col cols="6">
          <Bar
            :options="{
              indexAxis: 'y',
            }"
            :data="tagChartData"
          ></Bar>
          Show tags with a count of at least <v-text-field v-model="cutoff"></v-text-field>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped>
.color-opts {
  display: flex;
  height: 50px;
}

.color-opt {
  cursor: pointer;
  width: 100px;
}
</style>
