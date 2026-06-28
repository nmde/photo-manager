<script setup lang="ts">
  import type { LayerRec } from '@/classes/Layer';
  import type { PlaceRec } from '@/classes/Place';
  import { get_layers, get_places } from '@/api/places';

  const props = defineProps<{
    id?: string;
    loading?: boolean;
    value: string[];
  }>();

  const emit = defineEmits<{
    (e: 'focused', val: boolean): void;
    (e: 'update', val: string[]): void;
  }>();

  const layers = ref<LayerRec>({});
  const placeList = ref<PlaceRec>({});

  async function initialize() {
    await get_layers()
      .ok(l => (layers.value = l))
      .err(reportError)
      .send();
    await get_places()
      .ok(p => (placeList.value = p))
      .err(reportError)
      .send();
  }

  watch(() => props.id, initialize);

  onMounted(async () => {
    await initialize();
  });
</script>

<template>
  <sorted-combo
    :id="id"
    color-key="layer"
    :color-repo="layers"
    :items="placeList"
    label="Location"
    :value="value"
    @focused="val => emit('focused', val)"
    @update="location => emit('update', location)"
  />
</template>
