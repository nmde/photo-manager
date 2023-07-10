import { createRouter, createWebHistory } from 'vue-router';
import CollectionView from '../views/CollectionView.vue';
import LandingView from '../views/LandingView.vue';
import TaggerView from '../views/TaggerView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'landing',
      component: LandingView,
    },
    {
      path: '/collection',
      name: 'collection',
      component: CollectionView,
    },
    {
      path: '/tagger',
      name: 'tagger',
      component: TaggerView,
    },
  ],
});

export default router;
