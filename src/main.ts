import { createApp } from 'vue';
import '@mdi/font/css/materialdesignicons.css';
import 'vuetify/styles';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { createMemoryHistory, createRouter } from 'vue-router';
import App from './App.vue';
import { routes } from './routes';

createApp(App)
  .use(createVuetify({ components, directives }))
  .use(createRouter({ history: createMemoryHistory(), routes }))
  .mount('#app');
