import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";

import '@mdi/font/css/materialdesignicons.css';
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { createVuetify } from 'vuetify'

const vuetify = createVuetify({
	components,
	directives,
	// Vuetify 4 defaults to `system`; keep this app in light mode.
	theme: {
		defaultTheme: 'light',
	},
})

createApp(App).use(vuetify).mount("#app");
