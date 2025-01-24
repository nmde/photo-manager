import { RouteRecordRaw } from 'vue-router';
import Index from './pages/index.vue';
import Tagger from './pages/tagger.vue';
import Tags from './pages/tags.vue';
import Stats from './pages/stats.vue';
import Settings from './pages/settings.vue';
import People from './pages/people.vue';
import Locations from './pages/locations.vue';
import Journal from './pages/journal.vue';
import Calendar from './pages/calendar.vue';

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: Index,
  },
  {
    path: '/calendar',
    component: Calendar,
  },
  {
    path: '/journal',
    component: Journal,
  },
  {
    path: '/locations',
    component: Locations,
  },
  {
    path: '/people',
    component: People,
  },
  {
    path: '/settings',
    component: Settings,
  },
  {
    path: '/stats',
    component: Stats,
  },
  {
    path: '/tagger',
    component: Tagger,
  },
  {
    path: '/tags',
    component: Tags,
  },
];
