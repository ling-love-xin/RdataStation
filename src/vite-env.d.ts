/// <reference types="vite/client" />

declare module 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution'

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
