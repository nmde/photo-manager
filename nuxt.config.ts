import { defineNuxtConfig } from 'nuxt/config';
import { aliases, mdi } from 'vuetify/iconsets/mdi';

export default defineNuxtConfig({
  devtools: { enabled: true },
  modules: ['@invictus.codes/nuxt-vuetify', '@pinia/nuxt'],
  srcDir: 'src',
  vuetify: {
    vuetifyOptions: {
      icons: {
        defaultSet: 'mdi',
        aliases,
        sets: {
          mdi,
        },
      },
    },
    moduleOptions: {
      useVuetifyLabs: true,
    },
  },
  pinia: {
    autoImports: ['defineStore'],
  },
});
