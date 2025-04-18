import { createApp } from "vue";
import App from "./App.vue";
import { vuetify } from './plugins/vuetify'
import './assets/style.css';
import './assets/css/animations.css';

createApp(App).use(vuetify).mount("#app");
