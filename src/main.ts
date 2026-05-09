import { createApp } from 'vue';
import { registerPlugins } from '@/plugins';
import App from './App.vue';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import './styles/tokens.css';
import './styles/styles.css';

const app = createApp(App);

registerPlugins(app);

app.mount('#app');
