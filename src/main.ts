import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from './router'
import { vuetify } from './plugins/vuetify'
import Tres from '@tresjs/core'
import i18n from './i18n'
import './assets/fonts.css';
import './assets/style.css';
import './assets/css/animations.css';
import './assets/theme.css';
import './assets/styles/console-ui.css';
import './assets/styles/console-dialog.css';

const pinia = createPinia();
const app = createApp(App);

app.use(pinia).use(router).use(vuetify).use(Tres).use(i18n);
app.mount("#app");
