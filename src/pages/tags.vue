<script setup lang="ts">
  import type { TagRec } from '@/classes/Tag';
  import { stringSimilarity } from 'string-similarity-js';
  import { get_tags } from '@/api/tags';
  import { useFileStore } from '@/stores/fileStore';

  const route = useRoute();
  const router = useRouter();
  const { reportError } = useFileStore();

  const selected = ref<string[]>([]);
  const tags = ref<TagRec>({});
  const query = ref('');
  const graphContainer = useTemplateRef('graphContainer');
  const graphWidth = ref(0);
  const graphHeight = ref(0);
  const changeCounter = ref(0);
  const showGraph = ref(false);

  const sortedTags = computed(() => {
    const t = Object.values(tags.value).toSorted((a, b) => b.count - a.count);
    return query.value.length > 1
      ? t
        .map(x => ({ value: x, score: stringSimilarity(query.value, x.name) }))
        .filter(x => x.score > 0)
        .toSorted((a, b) => b.score - a.score)
        .map(x => x.value)
      : t;
  });

  const selectedTag = computed(() => tags.value[selected.value[0] ?? '']);

  onMounted(async () => {
    await get_tags()
      .ok(t => (tags.value = t))
      .err(message => reportError(message))
      .send();
    if (typeof route.query.tag === 'string') {
      selected.value[0] = route.query.tag;
    }
    if (graphContainer.value) {
      graphWidth.value = graphContainer.value.offsetWidth;
      graphHeight.value = graphContainer.value.offsetHeight;
    }
  });
</script>

<template>
  <v-container class="tag-page" fluid>
    <v-row no-gutters>
      <v-col>
        <v-toolbar color="primary">
          <div class="toolbar-controls">
            <v-text-field v-model="query" clearable label="Filter Tags" />
          </div>
        </v-toolbar>
        <div class="tag-list">
          <v-list item-title="name" :items="sortedTags">
            <template #item="{ props: iprops }">
              <v-list-item
                v-bind="iprops"
                :active="selected[0] === iprops.title"
                :base-color="tags[iprops.title]?.color ?? undefined"
                :title="`${iprops.title} (${tags[iprops.title]?.count})`"
                @click="selected[0] = iprops.title"
              />
            </template>
          </v-list>
        </div>
      </v-col>
      <v-col cols="9">
        <div class="tag-details">
          <h2 :style="{ color: selectedTag?.color ?? 'inherit' }">
            {{ selectedTag?.name ?? 'Select a tag' }}
          </h2>
          <v-btn
            v-if="selectedTag"
            color="primary"
            @click="router.push(`/tagger?tag=${selectedTag.name}`)"
          >
            View Photos ({{ selectedTag.count ?? 0 }})
          </v-btn>
          <br />
          <br />
          Set Tag Color:
          <color-options
            :disabled="selectedTag === undefined"
            @select="
              color => {
                if (selectedTag !== undefined) {
                  selectedTag.setColor(color ?? null);
                }
              }
            "
          />
          <br />
          <tag-input
            :disabled="selectedTag === undefined"
            label="Prerequisite Tags"
            :value="selectedTag?.prereqs ?? []"
            @change="
              async tags => {
                await selectedTag?.setPrereqs(tags);
                changeCounter += 1;
              }
            "
          />
          <tag-input
            :disabled="selectedTag === undefined"
            label="Corequisite Tags"
            :value="selectedTag?.coreqs ?? []"
            @change="
              async tags => {
                await selectedTag?.setCoreqs(tags);
                changeCounter += 1;
              }
            "
          />
          <tag-input
            :disabled="selectedTag === undefined"
            label="Incompatible Tags"
            :value="selectedTag?.incompatible ?? []"
            @change="
              async tags => {
                await selectedTag?.setIncompatible(tags);
                changeCounter += 1;
              }
            "
          />
          <v-btn @click="showGraph = !showGraph">Toggle Graph</v-btn>
        </div>
        <div ref="graphContainer" class="fill-height">
          <tag-graph
            v-if="showGraph"
            :changed="changeCounter"
            :data="tags"
            :height="graphHeight"
            :width="graphWidth"
          />
        </div>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
  .tag-page {
    margin: 0;
  }

  .tag-list {
    max-height: 92vh;
    overflow-y: scroll;
  }

  .tag-details {
    margin: var(--space-md);
  }

  .tag-details h2 {
    font-size: var(--text-size-headline);
    font-weight: var(--text-weight-bold);
    line-height: var(--text-lh-tight);
    letter-spacing: var(--text-tracking-tight);
    margin-bottom: var(--space-sm);
  }
</style>
