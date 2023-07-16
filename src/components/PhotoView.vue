<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { watch, ref } from 'vue';
import { Photo } from '../classes/Photo';
import { useFileStore } from '../stores/fileStore';

const props = defineProps<{
  modelValue: Photo;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: Photo): void;
}>();

const fileStore = useFileStore();
const { addTags } = fileStore;
const { tags } = storeToRefs(fileStore);

const title = ref('');
const description = ref('');
const locationApprox = ref(false);
const isDuplicate = ref(false);
const selectedTags = ref<string[]>([]);

/**
 * Adds new tags to the master list.
 */
function updateTags() {
  selectedTags.value.forEach((tag) => {
    if (tags.value.indexOf(tag) < 0) {
      addTags(tag);
    }
  });
  emit('update:modelValue', {
    name: props.modelValue.name,
    title: title.value,
    description: description.value,
    path: props.modelValue.path,
    locationApprox: locationApprox.value,
    tags: selectedTags.value,
    isDuplicate: isDuplicate.value,
  });
}

watch(props.modelValue, () => {
  title.value = props.modelValue.title;
  description.value = props.modelValue.description;
  selectedTags.value = props.modelValue.tags;
  locationApprox.value = props.modelValue.locationApprox;
  isDuplicate.value = props.modelValue.isDuplicate;
});
</script>

<template>
  <v-card>
    <v-card-title>{{ props.modelValue.name }}</v-card-title>
    <v-card-text>
      <v-img max-height="600" :src="props.modelValue.path"></v-img>
      <v-text-input label="Title" v-model="title"></v-text-input>
      <v-textarea label="Description" v-model="description"></v-textarea>
      <v-combobox
        label="Photo Tags"
        :items="tags"
        multiple
        chips
        v-model="selectedTags"
        @update:model-value="updateTags"
      ></v-combobox>
      <v-checkbox label="Location is approximate" v-model="locationApprox"></v-checkbox>
      <v-checkbox label="Mark as duplicate" v-model="isDuplicate"></v-checkbox>
    </v-card-text>
  </v-card>
</template>
