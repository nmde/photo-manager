import { createApp } from 'vue';
import '@mdi/font/css/materialdesignicons.css';
import 'vuetify/styles';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { VDateInput } from 'vuetify/labs/VDateInput';
import { VCalendar } from 'vuetify/labs/VCalendar';
import { createMemoryHistory, createRouter } from 'vue-router';
import App from './App.vue';
import { routes } from './routes';

createApp(App)
  .use(
    createVuetify({
      components: {
        ...components,
        VDateInput,
        VCalendar,
      },
      directives,
    }),
  )
  .use(createRouter({ history: createMemoryHistory(), routes }))
  .mount('#app');
