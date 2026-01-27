import { createApp } from "vue";
import App from "./App.vue";
import { vuetify } from './plugins/vuetify'
import i18n from './i18n'
import './assets/fonts.css';
import './assets/style.css';
import './assets/css/animations.css';
import './assets/theme.css';

createApp(App).use(vuetify).use(i18n).mount("#app");
