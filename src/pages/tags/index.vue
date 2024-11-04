<script setup lang="ts">
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale, Tooltip } from 'chart.js';
import { Bar } from 'vue-chartjs';
import { fileStore } from '../../stores/fileStore';
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

const {
  setTagColor,
  setTagPrereqs,
  setTagIncompatible,
  getTagColor,
  setTagCoreqs,
  handleTagChange,
  tagCounts,
  advTags,
  files,
} = fileStore;

const cutoff = ref(1);
const selected = ref('');
const selectedColor = ref('black');
const prereqTags = ref<string[]>([]);
const coreqTags = ref<string[]>([]);
const incompatibleTags = ref<string[]>([]);
const filterColor = ref('');
const relative = ref(false);

const avgRating = computed(() => {
  let sum = 0;
  let count = 0;
  Object.values(files).forEach((photo) => {
    if (photo.rating) {
      sum += photo.rating;
      count += 1;
    }
  });
  return sum / count;
});

const avgTags = computed(() => {
  let sum = 0;
  let count = 0;
  Object.values(files).forEach((photo) => {
    sum += photo.tags.length;
    count += 1;
  });
  return sum / count;
});

const tagChartData = computed(() => {
  let sorted: string[] = [];
  const backgroundColor: string[] = [];
  Object.entries(tagCounts)
    .filter((count) => count[1] >= cutoff.value)
    .forEach(([tag, value]) => {
      let color = getTagColor(tag);
      if (color === 'black' || color.length === 0) {
        color = 'rgba(201, 203, 207, 0.8)';
      }
      if (filterColor.value.length > 0 && color !== filterColor.value) {
        return;
      }
      if (sorted.length === 0) {
        sorted.push(tag);
        backgroundColor.push(color);
      } else {
        let i = 0;
        while (i < sorted.length && value < tagCounts[sorted[i]]) {
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
        data: sorted.map((tag) => tagCounts[tag]),
        backgroundColor,
      },
    ],
  };
});

const tagRatingData = computed(() => {
  let sorted: string[] = [];
  const ratingsMap: Record<string, number[]> = {};
  Object.entries(tagCounts)
    .filter((count) => count[1] >= cutoff.value)
    .forEach(([tag]) => {
      let color = getTagColor(tag);
      if (color === 'black' || color.length === 0) {
        color = 'rgba(201, 203, 207, 0.8)';
      }
      if (filterColor.value.length > 0 && color !== filterColor.value) {
        return;
      }
      sorted.push(tag);
      ratingsMap[tag] = [0, 0];
      Object.values(files)
        .filter((photo) => photo.hasTag(tag))
        .forEach((photo) => {
          if (photo.rating) {
            ratingsMap[tag][0] += 1;
            ratingsMap[tag][1] += photo.rating;
          }
        });
    });
  sorted = sorted
    .filter((tag) => ratingsMap[tag][0] >= cutoff.value)
    .sort((a, b) => {
      let aa = ratingsMap[a][1] / ratingsMap[a][0];
      let ba = ratingsMap[b][1] / ratingsMap[b][0];
      if (relative.value) {
        aa -= avgRating.value;
        ba -= avgRating.value;
      }
      if (aa > ba) {
        return -1;
      }
      if (aa < ba) {
        return 1;
      }
      return 0;
    });
  return {
    labels: sorted,
    datasets: [
      {
        axis: 'y',
        label: 'Avg. Rating',
        data: sorted.map((tag) => {
          let avg = ratingsMap[tag][1] / ratingsMap[tag][0];
          if (relative.value) {
            avg -= avgRating.value;
          }
          return avg;
        }),
        backgroundColor: sorted.map((tag) => {
          let color = getTagColor(tag);
          if (color === 'black' || color.length === 0) {
            color = 'rgba(201, 203, 207, 0.8)';
          }
          return color;
        }),
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
                selected = tags as unknown as string;
                const adv = (advTags as Tag[]).find((t) => t.data.name === selected);
                selectedColor = getTagColor(selected);
                console.log(adv);
                if (adv) {
                  prereqTags = adv.prereqs;
                  coreqTags = adv.coreqs;
                  incompatibleTags = adv.incompatible;
                } else {
                  prereqTags = [];
                  coreqTags = [];
                  incompatibleTags = [];
                }
              }
            "
          ></tag-input>
          <div v-if="selected">
            Editing properties of <span :style="{ color: selectedColor }">{{ selected }}</span>
            <br />
            Set color:
            <color-options
              @select="
                async (color) => {
                  selectedColor = color;
                  await setTagColor(selected, color);
                }
              "
            ></color-options>
            <br />
            <tag-input
              label="Prerequisite Tags"
              :value="prereqTags"
              @update="
                async (tags) => {
                  await setTagPrereqs(selected, tags);
                  handleTagChange(selected);
                }
              "
            ></tag-input>
            <tag-input
              label="Corequisite Tags"
              :value="coreqTags"
              @update="
                async (tags) => {
                  await setTagCoreqs(selected, tags);
                  handleTagChange(selected);
                }
              "
            ></tag-input>
            <tag-input
              label="Incompatible Tags"
              :value="incompatibleTags"
              @update="
                async (tags) => {
                  await setTagIncompatible(selected, tags);
                  handleTagChange(selected);
                }
              "
            ></tag-input>
          </div>
        </v-col>
        <v-col cols="6">
          <!-- TODO: this should be one graph with multiple bars / sorting options -->
          <Bar
            :options="{
              indexAxis: 'y',
            }"
            :data="tagChartData"
          ></Bar>
          <Bar
            :options="{
              indexAxis: 'y',
            }"
            :data="tagRatingData"
          ></Bar>
          <v-checkbox label="Show relative rating impact" v-model="relative"></v-checkbox>
          Show tags with a count of at least <v-text-field v-model="cutoff"></v-text-field>
          Filter by color:
          <color-options @select="(color) => (filterColor = color)"></color-options>
          Average tags per photo: {{ avgTags.toPrecision(3) }}<br />
          Overall average rating: {{ avgRating.toPrecision(3) }}
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>
