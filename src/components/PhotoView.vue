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
const { updateTags } = fileStore;
const { tags } = storeToRefs(fileStore);

const title = ref('');
const description = ref('');
const locationApprox = ref(false);
const isDuplicate = ref(false);
const selectedTags = ref<string[]>([]);

/**
 * Adds new tags to the master list.
 */
function syncTags() {
  updateTags(props.modelValue.data.name, selectedTags.value);
  emit(
    'update:modelValue',
    new Photo({
      ...props.modelValue.data,
      title: title.value,
      description: description.value,
      locationApprox: locationApprox.value,
      tags: selectedTags.value.join(','),
      isDuplicate: isDuplicate.value,
    }),
  );
}

watch(props.modelValue, () => {
  title.value = props.modelValue.data.title;
  description.value = props.modelValue.data.description;
  selectedTags.value = props.modelValue.tags;
  locationApprox.value = props.modelValue.data.locationApprox;
  isDuplicate.value = props.modelValue.data.isDuplicate;
});
</script>

<template>
  <v-card>
    <v-card-title>{{ props.modelValue.data.name }}</v-card-title>
    <v-card-text>
      <v-img max-height="600" :src="props.modelValue.data.path"></v-img>
      <v-text-input label="Title" v-model="title"></v-text-input>
      <v-textarea label="Description" v-model="description"></v-textarea>
      <v-combobox
        label="Photo Tags"
        :items="tags"
        multiple
        chips
        v-model="selectedTags"
        @update:model-value="syncTags"
      ></v-combobox>
      <v-checkbox label="Location is approximate" v-model="locationApprox"></v-checkbox>
      <v-checkbox label="Mark as duplicate" v-model="isDuplicate"></v-checkbox>
    </v-card-text>
  </v-card>
</template>
