import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { nextTick, ref, Teleport } from 'vue'

import CommandPalette from './CommandPalette.vue'
import { useCommandStore } from '../../stores/command-store'

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}))

describe('CommandPalette', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  function mountWithTeleport(props: { visible: boolean }) {
    const visibleRef = ref(props.visible)
    const onClose = vi.fn(() => {
      visibleRef.value = false
    })

    const wrapper = mount(
      {
        components: { CommandPalette },
        template: `
        <div id="app">
          <CommandPalette :visible="visible" @close="onClose" />
        </div>
        <div id="teleport-target"></div>
      `,
        setup() {
          return {
            visible: visibleRef,
            onClose,
          }
        },
      },
      {
        attachTo: document.body,
        global: {
          components: { Teleport },
        },
      }
    )

    return { wrapper, visibleRef, onClose }
  }

  it('should display registered commands', async () => {
    const store = useCommandStore()
    store.register({
      id: 'test-cmd',
      label: 'Test Command',
      category: 'test',
      action: vi.fn(),
    })

    mountWithTeleport({ visible: true })
    await nextTick()

    expect(document.body.textContent).toContain('Test Command')
  })

  it('should filter commands by search query', async () => {
    const store = useCommandStore()
    store.register({ id: 'new-query', label: 'New Query', category: 'file', action: vi.fn() })
    store.register({ id: 'open-project', label: 'Open Project', category: 'file', action: vi.fn() })

    mountWithTeleport({ visible: true })
    await nextTick()

    const input = document.querySelector('.command-palette-input') as HTMLInputElement
    input.value = 'new'
    input.dispatchEvent(new Event('input'))
    await nextTick()

    expect(document.body.textContent).toContain('New Query')
    expect(document.body.textContent).not.toContain('Open Project')
  })

  it('should execute command on click', async () => {
    const store = useCommandStore()
    const action = vi.fn()
    store.register({
      id: 'test-cmd',
      label: 'Test Command',
      category: 'test',
      action,
    })

    mountWithTeleport({ visible: true })
    await nextTick()

    const commandItem = document.querySelector('.command-item') as HTMLElement
    commandItem.click()
    await nextTick()

    expect(action).toHaveBeenCalledTimes(1)
  })

  it('should emit close event on escape key', async () => {
    const { onClose } = mountWithTeleport({ visible: true })
    await nextTick()

    const input = document.querySelector('.command-palette-input') as HTMLInputElement
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await nextTick()

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should emit close event on overlay click', async () => {
    const { onClose } = mountWithTeleport({ visible: true })
    await nextTick()

    const overlay = document.querySelector('.command-palette-overlay') as HTMLElement
    overlay.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await nextTick()

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should navigate with arrow keys', async () => {
    const store = useCommandStore()
    store.register({ id: 'cmd1', label: 'Command 1', category: 'test', action: vi.fn() })
    store.register({ id: 'cmd2', label: 'Command 2', category: 'test', action: vi.fn() })

    mountWithTeleport({ visible: true })
    await nextTick()

    const input = document.querySelector('.command-palette-input') as HTMLInputElement
    const items = document.querySelectorAll('.command-item')

    // First item should be active by default
    expect(items[0].classList.contains('active')).toBe(true)

    // Press down arrow
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }))
    await nextTick()
    expect(items[1].classList.contains('active')).toBe(true)

    // Press up arrow
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp' }))
    await nextTick()
    expect(items[0].classList.contains('active')).toBe(true)
  })

  it('should execute command on enter key', async () => {
    const store = useCommandStore()
    const action = vi.fn()
    store.register({
      id: 'test-cmd',
      label: 'Test Command',
      category: 'test',
      action,
    })

    const { onClose } = mountWithTeleport({ visible: true })
    await nextTick()

    const input = document.querySelector('.command-palette-input') as HTMLInputElement
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    await nextTick()

    expect(action).toHaveBeenCalledTimes(1)
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should display no results message when search has no matches', async () => {
    mountWithTeleport({ visible: true })
    await nextTick()

    const input = document.querySelector('.command-palette-input') as HTMLInputElement
    input.value = 'xyz-nonexistent'
    input.dispatchEvent(new Event('input'))
    await nextTick()

    expect(document.body.textContent).toContain('commandPalette.noResults')
  })
})
