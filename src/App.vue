<script setup lang="ts">
  import { useFileStore } from './stores/fileStore';

  const route = useRoute();

  const store = useFileStore();
  const { theme, globalError } = storeToRefs(store);

  const errorSnack = ref(false);

  watch(globalError, () => {
    errorSnack.value = true;
  });
</script>

<template>
  <v-app :theme="theme === 'Dark' ? 'Theme' : 'LightTheme'">
    <v-layout>
      <v-navigation-drawer v-if="route.path !== '/'" color="secondary" expand-on-hover permanent rail>
        <v-list class="top-nav" color="primary" nav>
          <v-list-item prepend-icon="mdi-image" title="Photos" to="/tagger" />
          <v-list-item prepend-icon="mdi-map-marker" title="Locations" to="/locations" />
          <v-list-item prepend-icon="mdi-calendar" title="Calendar" to="/calendar" />
          <v-divider />
          <v-list-item prepend-icon="mdi-tag" title="Tags" to="/tags" />
          <v-list-item prepend-icon="mdi-account" title="People" to="/people" />
          <v-list-item prepend-icon="mdi-chart-line" title="Statistics" to="/stats" />
        </v-list>
        <v-spacer />
        <v-list color="primary" nav>
          <v-list-item prepend-icon="mdi-cog" title="Settings" to="/settings" />
          <v-list-item prepend-icon="mdi-exit-to-app" title="Close Project" to="/" />
        </v-list>
      </v-navigation-drawer>
      <v-main>
        <RouterView v-slot="{ Component }">
          <Transition mode="out-in" name="route">
            <div :key="$route.path" class="route-view">
              <component :is="Component" />
            </div>
          </Transition>
        </RouterView>
      </v-main>
    </v-layout>
    <v-snackbar v-model="errorSnack" color="error">
      {{ globalError }}
    </v-snackbar>
  </v-app>
</template>

<style scoped>
  .top-nav {
    height: calc(100vh - 128px);
  }

  .route-view {
    height: 100%;
  }
</style>
