import { ref, computed } from 'vue'

export interface AccessibilityConfig {
  highContrast: boolean
  reducedMotion: boolean
  fontSize: 'small' | 'medium' | 'large'
  screenReaderOptimized: boolean
}

const STORAGE_KEY = 'navigator:a11y'

export function useAccessibility() {
  const config = ref<AccessibilityConfig>({
    highContrast: false,
    reducedMotion: false,
    fontSize: 'medium',
    screenReaderOptimized: false
  })

  const fontSizeClass = computed(() => {
    switch (config.value.fontSize) {
      case 'small':
        return 'font-size-small'
      case 'large':
        return 'font-size-large'
      default:
        return 'font-size-medium'
    }
  })

  const highContrastClass = computed(() =>
    config.value.highContrast ? 'high-contrast' : ''
  )

  function loadConfig() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY)
      if (saved) {
        config.value = { ...config.value, ...JSON.parse(saved) }
      }
    } catch (e) {
      console.error('Failed to load accessibility config:', e)
    }

    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      config.value.reducedMotion = true
    }

    if (window.matchMedia('(prefers-contrast: more)').matches) {
      config.value.highContrast = true
    }
  }

  function saveConfig() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(config.value))
    } catch (e) {
      console.error('Failed to save accessibility config:', e)
    }
  }

  function updateConfig(partial: Partial<AccessibilityConfig>) {
    config.value = { ...config.value, ...partial }
    saveConfig()
  }

  function getAriaLabel(
    type: string,
    name: string,
    additional?: string
  ): string {
    const base = `${type}: ${name}`
    return additional ? `${base} - ${additional}` : base
  }

  function getTreeItemAriaAttributes(
    type: string,
    name: string,
    hasChildren: boolean,
    isExpanded: boolean,
    isSelected: boolean
  ): Record<string, string | boolean> {
    return {
      'aria-label': getAriaLabel(type, name, `${isExpanded ? '已展开' : '已折叠'}, ${isSelected ? '已选中' : '未选中'}`),
      'aria-expanded': isExpanded,
      'aria-selected': isSelected,
      'aria-level': '1',
      role: 'treeitem'
    }
  }

  function getButtonAriaAttributes(
    label: string,
    disabled: boolean = false,
    additional?: string
  ): Record<string, string | boolean> {
    return {
      'aria-label': additional ? `${label} - ${additional}` : label,
      'aria-disabled': disabled,
      role: 'button'
    }
  }

  function getLiveRegionAttributes(message: string): Record<string, string> {
    return {
      'aria-live': 'polite',
      'aria-atomic': 'true',
      role: 'status'
    }
  }

  loadConfig()

  return {
    config,
    fontSizeClass,
    highContrastClass,
    updateConfig,
    getAriaLabel,
    getTreeItemAriaAttributes,
    getButtonAriaAttributes,
    getLiveRegionAttributes
  }
}
