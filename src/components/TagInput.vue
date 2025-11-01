<script setup lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { computed, ref, watch } from 'vue';
  import { Graph } from '@/classes/Graph';
  import { Tag, type TagData } from '@/classes/Tag';
  import { fileStore } from '../stores/fileStore';

  const { getFile, advTags } = fileStore;

  const props = defineProps<{
    label: string;
    value: string[];
    single?: boolean;
    filtered?: boolean;
    validate?: string;
    advanced?: boolean;
    target?: string;
    loading?: boolean;
  }>();

  const emit = defineEmits<{
    (e: 'change' | 'update', tags: string[]): void;
  }>();

  const selected = ref<Tag[]>([]);
  const valid = ref<boolean | undefined>(true);
  const validationMsg = ref<string | undefined>(undefined);
  const hasChanged = ref(false);
  const tags = ref<Tag[]>([]);

  const targetPhoto = computed(() => (props.target ? getFile(props.target) : undefined));

  async function initialize() {
    selected.value = props.value
      .map(t => tags.value.find(tag => tag.name === t))
      .filter(t => t !== undefined);
    const allTags = Tag.createTags(
      Object.values(await invoke<Record<string, TagData>>('get_tags')),
    );
    if (props.filtered) {
      const tagGraph = new Graph<Tag>();
      for (const tag of allTags) {
        if (props.value.includes(tag.name)) {
          // Always show tags that are already enabled regardless of prereqs
          tagGraph.add(tag.name, tag);
          continue;
        }
        const a = advTags.find(t => t.name === tag.name);
        if (a) {
          if (a.prereqs.length > 0) {
            let anyPrereqMet = false;
            for (const p of a.prereqs) {
              anyPrereqMet = anyPrereqMet || props.value.includes(p);
            }
            if (anyPrereqMet) {
              tagGraph.add(tag.name, tag);
            }
          } else {
            tagGraph.add(tag.name, tag);
          }
        } else {
          tagGraph.add(tag.name, tag);
        }
      }
      tags.value = tagGraph.toSorted().map(node => node.data);
    } else {
      tags.value = allTags;
    }
  }

  watch(() => props.value, initialize);

  fileStore.on('validationUpdate', () => {
    valid.value = targetPhoto.value?.valid;
    validationMsg.value = targetPhoto.value?.validationMsg;
  });
</script>

<template>
  <v-combobox
    v-model="selected"
    chips
    clearable
    :error="!valid"
    :error-messages="validationMsg"
    item-title="name"
    item-value="name"
    :items="tags"
    :label="props.label"
    :loading="loading"
    :multiple="props.single ? false : true"
    @update:focused="
      () => {
        if (hasChanged) {
          emit(
            'update',
            selected.map(t => t.name),
          );
        }
      }
    "
    @update:model-value="
      () => {
        hasChanged = true;
        emit(
          'change',
          selected.map(t => t.name),
        );
      }
    "
  >
    <template #item="{ item, props: lprops }">
      <v-list-item v-bind="lprops" :style="{ color: item.raw.color }" :title="item.title" />
    </template>
    <template #chip="{ item, props: cprops }">
      <v-chip v-bind="cprops" :color="tags.find(t => t.name === item.value)?.color" />
    </template>
  </v-combobox>
  <div v-if="advanced" />
</template>
