<script setup lang="ts">
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const { searchHistory } = storeToRefs(store);

  const query = defineModel<string[]>({ required: true });
  const focused = defineModel<boolean>('focused', { required: true });
  const searching = defineModel<boolean>('searching', { required: true });
  const refreshing = defineModel<boolean>('refreshing', { required: true });

  const tmpText = ref<string[]>([]);
  const atDialog = ref(false);

  const emit = defineEmits<{
    (e: 'search' | 'refresh'): void;
  }>();

  function search() {
    if (query.value.length > 0) {
      store.pushHistory(query.value);
    }
    emit('search');
  }

  function applyHistory(entry: string[]) {
    query.value = entry;
    store.pushHistory(entry);
    emit('search');
  }

  // Everything that gets displayed in the dropdown beneath the search bar
  const items = computed<
    {
      type: string;
      value: string[];
    }[]
  >(() => [
    ...searchHistory.value.map(entry => ({
      type: 'history',
      value: entry,
    })),
    {
      type: 'at',
      value: ['Location: '],
    },
  ]);
</script>

<template>
  <v-combobox
    v-model="query"
    aria-autocomplete="none"
    chips
    clearable
    density="compact"
    :items="items"
    label="Search"
    multiple
    variant="outlined"
    @update:focused="val => (focused = val)"
  >
    <template #append>
      <v-menu v-if="searchHistory.length > 0">
        <template #activator="{ props: menuProps }">
          <v-btn v-bind="menuProps" density="compact" icon>
            <v-icon>mdi-history</v-icon>
          </v-btn>
        </template>
      </v-menu>
      <v-btn density="compact" icon :loading="searching" @click="search()">
        <v-icon>mdi-magnify</v-icon>
      </v-btn>
      <v-btn density="compact" icon :loading="refreshing" @click="emit('refresh')">
        <v-icon>mdi-refresh</v-icon>
      </v-btn>
    </template>
    <template #item="{ props, item }">
      <template v-if="item.type === 'history'">
        <v-list-item v-bind="props" title="" @click="applyHistory(item.value)">
          <template #prepend>
            <v-icon>mdi-history</v-icon>
          </template>
          <v-chip-group>
            <v-chip v-for="(term, i) in item.value" :key="i">
              {{ term }}
            </v-chip>
          </v-chip-group>
        </v-list-item>
      </template>
      <v-list-item v-else-if="item.type === 'at'" @click="atDialog = true">
        Photo Location:
      </v-list-item>
    </template>
  </v-combobox>
  <form-dialog
    v-model="atDialog"
    :reset="() => (tmpText = [])"
    size="sm"
    title="Photo Location"
    @submit="
      () => {
        query.push(`at:${tmpText[0]}`);
      }
    "
  >
    <location-input :value="tmpText" @update="location => (tmpText = location)" />
  </form-dialog>
</template>
