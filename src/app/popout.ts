import { DockviewVue } from 'dockview-vue'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

import 'dockview-vue/dist/styles/dockview.css'

const app = createApp({
  components: { DockviewVue },
  template: `<DockviewVue class="dockview" />`
})

app.use(createPinia())

app.mount('#popout-root')