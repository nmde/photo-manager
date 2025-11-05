<script setup lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { BarElement, CategoryScale, Chart as ChartJS, LinearScale, Tooltip } from 'chart.js';
  import { computed, ref } from 'vue';
  import { Bar } from 'vue-chartjs';
  import { Tag } from '../classes/Tag';

  /**
   * TODO:
   * - Delete a tag
   * - View redundant tags
   * - Tag network
   */

  ChartJS.register(CategoryScale, LinearScale, BarElement, Tooltip);

  const cutoff = ref(30);
  const selected = ref<Tag | undefined>();
  const filterColor = ref('');
  const relative = ref(false);
  const showGraphs = ref(false);
  const avgTags = ref(0);
  const avgRating = ref(0);

  const tagChartData = computed(() => {
    const sorted: string[] = [];
    const backgroundColor: string[] = [];
    for (const [tag, value] of Object.entries(tagCounts).filter(
      count => count[1] >= cutoff.value,
    )) {
      let color = getTagColor(tag);
      if (color === 'black' || color.length === 0) {
        color = 'rgba(201, 203, 207, 0.8)';
      }
      if (filterColor.value.length > 0 && color !== filterColor.value) {
        continue;
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
    }
    return {
      labels: sorted,
      datasets: [
        {
          axis: 'y',
          label: 'Count',
          data: sorted.map(tag => tagCounts[tag]),
          backgroundColor,
        },
      ],
    };
  });

  const tagRatingData = computed(() => {
    let sorted: string[] = [];
    const ratingsMap: Record<string, number[]> = {};
    for (const [tag] of Object.entries(tagCounts).filter(count => count[1] >= cutoff.value)) {
      let color = getTagColor(tag);
      if (color === 'black' || color.length === 0) {
        color = 'rgba(201, 203, 207, 0.8)';
      }
      if (filterColor.value.length > 0 && color !== filterColor.value) {
        continue;
      }
      sorted.push(tag);
      ratingsMap[tag] = [0, 0];
      for (const photo of Object.values(files).filter(photo => photo.hasTag(tag))) {
        if (photo.rating && ratingsMap[tag][0] && ratingsMap[tag][1]) {
          ratingsMap[tag][0] += 1;
          ratingsMap[tag][1] += photo.rating;
        }
      }
    }
    sorted = sorted
      .filter(tag => ratingsMap[tag]?.[0] && ratingsMap[tag][0] >= cutoff.value)
      .toSorted((a, b) => {
        let aa = ratingsMap[a]?.[1] / ratingsMap[a][0];
        let ba = ratingsMap[b]?.[1] / ratingsMap[b][0];
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
          data: sorted.map(tag => {
            let avg = ratingsMap[tag]?.[1] ?? 0 / (ratingsMap[tag]?.[0] ?? 1);
            if (relative.value) {
              avg -= avgRating.value;
            }
            return avg;
          }),
          backgroundColor: sorted.map(tag => {
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

  async function selectTag(tag: string) {
    selected.value
      = Tag.createTags(Object.values(await invoke<Record<string, Tag>>('get_tags'))).find(
        t => t.name === tag,
      ) ?? undefined;
  }

  onMounted(async () => {
    const { avg_count, avg_rating } = await invoke<{
      avg_count: number;
      avg_rating: number;
    }>('get_tag_stats');
    avgRating.value = avg_rating;
    avgTags.value = avg_count;
  });
</script>

<template>
  <v-main>
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <tag-input
            label="Select a tag"
            single
            :value="selected"
            @change="
              tags => {
                if (tags[0]) {
                  selectTag(tags[0]);
                }
              }
            "
          />
          <div v-if="selected">
            <br />
            Set color:
            <color-options @select="async color => await selected?.setColor(color)" />
            <br />
            <tag-input
              label="Prerequisite Tags"
              :value="selected.prereqs"
              @change="async tags => {
                console.log(tags);
                await selected?.setPrereqs(tags);
              }"
            />
            <tag-input
              label="Corequisite Tags"
              :value="selected.coreqs"
              @change="async tags => await selected?.setCoreqs(tags)"
            />
            <tag-input
              label="Incompatible Tags"
              :value="selected.incompatible"
              @change="async tags => await selected?.setIncompatible(tags)"
            />
          </div>
        </v-col>
        <v-col cols="6">
          <div v-if="showGraphs">
            <!-- TODO: this should be one graph with multiple bars / sorting options -->
            <Bar
              :data="tagChartData"
              :options="{
                indexAxis: 'y',
              }"
            />
            <Bar
              :data="tagRatingData"
              :options="{
                indexAxis: 'y',
              }"
            />
            <v-checkbox v-model="relative" label="Show relative rating impact" />
            Show tags with a count of at least <v-text-field v-model="cutoff" />
            Filter by color:
            <color-options @select="color => (filterColor = color)" />
          </div>
          <v-btn v-if="!showGraphs" @click="showGraphs = true">Show Graphs</v-btn>
          <br />
          Average tags per photo: {{ avgTags.toPrecision(3) }}
          <br />
          Overall average rating: {{ avgRating.toPrecision(3) }}
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>
