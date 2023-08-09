// Plugins
import VueVideoPlayer from '@videojs-player/vue';
import { createPinia } from 'pinia';
import 'video.js/dist/video-js.css';
import router from '../router';
import { loadFonts } from './webfontloader';
import vuetify from './vuetify';

// Types
import type { App } from 'vue';

export function registerPlugins(app: App) {
  loadFonts();
  app.use(vuetify);
  app.use(createPinia());
  app.use(router);
  app.use(VueVideoPlayer);
}
