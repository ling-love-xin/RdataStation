import type { editor } from 'monaco-editor'

const rdataDark: editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'keyword', foreground: '#E17055' },
    { token: 'operator.sql', foreground: '#E17055' },
    { token: 'identifier.sql', foreground: '#E5E7EB' },
  ],
  colors: {
    'editor.background': '#1E1F22',
    'editor.foreground': '#E5E7EB',
    'editor.lineHighlightBackground': '#2B2D30',
    'editor.selectionBackground': 'rgba(225, 112, 85, 0.25)',
    'editor.inactiveSelectionBackground': 'rgba(225, 112, 85, 0.15)',
    'editorCursor.foreground': '#E17055',
    'editorLineNumber.foreground': '#6B7280',
    'editorLineNumber.activeForeground': '#E5E7EB',
  },
}

const rdataLight: editor.IStandaloneThemeData = {
  base: 'vs',
  inherit: true,
  rules: [
    { token: 'keyword', foreground: '#E17055' },
    { token: 'operator.sql', foreground: '#E17055' },
    { token: 'identifier.sql', foreground: '#1F2937' },
  ],
  colors: {
    'editor.background': '#FFFFFF',
    'editor.foreground': '#1F2937',
    'editor.lineHighlightBackground': '#F5F5F5',
    'editor.selectionBackground': 'rgba(225, 112, 85, 0.15)',
    'editor.inactiveSelectionBackground': 'rgba(225, 112, 85, 0.08)',
    'editorCursor.foreground': '#E17055',
    'editorLineNumber.foreground': '#9CA3AF',
    'editorLineNumber.activeForeground': '#1F2937',
  },
}

export { rdataDark, rdataLight }
