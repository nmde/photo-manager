<script setup lang="ts">
import { fileStore } from '../stores/fileStore';

const { getTagColor, validateTags, getFile, tags, advTags } = fileStore;

const props = defineProps<{
  label: string;
  value: string | string[];
  single?: boolean;
  filtered?: boolean;
  validate?: string;
  advanced?: boolean;
  target?: string;
}>();

const emit = defineEmits<{
  (e: 'change', tags: string[]): void;
  (e: 'update', tags: string[]): void;
}>();

const selected = ref<string[]>([]);
const valid = ref<boolean | undefined>(true);
const validationMsg = ref<string | undefined>(undefined);

const targetPhoto = computed(() => {
  if (props.target) {
    return getFile(props.target);
  }
  return undefined;
});

const filteredTags = computed(() => {
  if (!props.filtered) {
    return tags;
  }
  const filtered: string[] = [];
  tags.forEach((tag) => {
    if (props.value.indexOf(tag) >= 0) {
      // Always show tags that are already enabled regardless of prereqs
      filtered.push(tag);
      return;
    }
    const a = advTags.find((t) => t.data.name === tag);
    if (a) {
      if (a.prereqs.length > 0) {
        let anyPrereqMet = false;
        a.prereqs.forEach((p) => {
          anyPrereqMet = anyPrereqMet || props.value.indexOf(p) >= 0;
        });
        if (anyPrereqMet) {
          filtered.push(tag);
        }
      } else {
        filtered.push(tag);
      }
    } else {
      filtered.push(tag);
    }
  });
  return filtered;
});

/**
 * Validates the tags.
 */
function validateTagsWrapper() {
  if (props.validate) {
    validateTags(props.validate);
  }
}

function initialize() {
  if (typeof props.value === 'string') {
    selected.value = [props.value];
  } else {
    selected.value = props.value;
  }
  if (props.validate) {
    validateTagsWrapper();
  }
}

watch(() => props.value, initialize);

fileStore.on('validationUpdate', () => {
  valid.value = targetPhoto.value?.valid;
  validationMsg.value = targetPhoto.value?.validationMsg;
});
</script>

// The global sorted tag list is not updating when new tags are added, links are created
<template>
  <v-combobox
    :label="props.label"
    :items="filteredTags"
    :multiple="props.single ? false : true"
    chips
    clearable
    v-model="selected"
    @update:model-value="
      () => {
        emit('change', selected);
        validateTagsWrapper();
      }
    "
    @update:focused="
      () => {
        emit('update', selected);
        validateTagsWrapper();
      }
    "
    :error="!valid"
    :error-messages="validationMsg"
  >
    <template v-slot:item="{ item, props }">
      <v-list-item v-bind="props" :style="{ color: getTagColor(item.title) }"></v-list-item>
    </template>
    <template v-slot:chip="{ item, props }">
      <v-chip v-bind="props" :color="getTagColor(item.title)"></v-chip>
    </template>
  </v-combobox>
  <div v-if="advanced"></div>
</template>
