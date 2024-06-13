import '@mdi/font/css/materialdesignicons.css';

import 'vuetify/styles';
import { createVuetify } from 'vuetify';
import { VDateInput } from 'vuetify/labs/VDateInput';
import { VTimePicker } from 'vuetify/labs/VTimePicker';
import { VCalendar } from 'vuetify/labs/VCalendar';

export default defineNuxtPlugin((app) => {
  const vuetify = createVuetify({
    components: {
      VCalendar,
      VDateInput,
      VTimePicker,
    },
  });
  app.vueApp.use(vuetify);
});
