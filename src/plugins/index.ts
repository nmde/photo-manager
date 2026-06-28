import type { App } from 'vue';
import { createRulesPlugin } from 'vuetify/labs/rules';
import { router } from '../router';
import { pinia } from '../stores';
import { vuetify } from './vuetify';

export function registerPlugins(app: App) {
  app.use(vuetify).use(createRulesPlugin({}, vuetify.locale)).use(router).use(pinia);
}
