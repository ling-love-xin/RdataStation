import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { nextTick } from 'vue'

import ProjectSelector from './ProjectSelector.vue'

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}))

describe('ProjectSelector', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  const defaultProps = {
    currentProject: 'Test Project',
    currentProjectId: 'proj-1',
    recentProjects: [
      { id: 'proj-1', name: 'Project 1', path: '/path/1' },
      { id: 'proj-2', name: 'Project 2', path: '/path/2' },
    ],
  }

  it('should render current project name', () => {
    const wrapper = mount(ProjectSelector, { props: defaultProps })
    expect(wrapper.text()).toContain('Test Project')
  })

  it('should render dropdown when clicked', async () => {
    const wrapper = mount(ProjectSelector, { props: defaultProps })
    await wrapper.find('.project-selector-btn').trigger('click')
    await nextTick()

    expect(wrapper.find('.project-dropdown').exists()).toBe(true)
    expect(wrapper.text()).toContain('Project 1')
    expect(wrapper.text()).toContain('Project 2')
  })

  it('should emit switch-project when clicking a recent project', async () => {
    const wrapper = mount(ProjectSelector, { props: defaultProps })
    await wrapper.find('.project-selector-btn').trigger('click')
    await nextTick()

    const projectItems = wrapper.findAll('.dropdown-item')
    await projectItems[1].trigger('click')

    expect(wrapper.emitted('switch-project')).toBeTruthy()
    expect(wrapper.emitted('switch-project')![0][0]).toEqual(defaultProps.recentProjects[1])
  })

  it('should emit new-project when clicking new project button', async () => {
    const wrapper = mount(ProjectSelector, { props: defaultProps })
    await wrapper.find('.project-selector-btn').trigger('click')
    await nextTick()

    await wrapper.find('.dropdown-actions button:first-child').trigger('click')

    expect(wrapper.emitted('new-project')).toBeTruthy()
  })

  it('should emit open-project when clicking open project button', async () => {
    const wrapper = mount(ProjectSelector, { props: defaultProps })
    await wrapper.find('.project-selector-btn').trigger('click')
    await nextTick()

    await wrapper.find('.dropdown-actions button:last-child').trigger('click')

    expect(wrapper.emitted('open-project')).toBeTruthy()
  })

  it('should close dropdown when clicking outside', async () => {
    const wrapper = mount(ProjectSelector, { props: defaultProps, attachTo: document.body })
    await wrapper.find('.project-selector-btn').trigger('click')
    await nextTick()

    expect(wrapper.find('.project-dropdown').exists()).toBe(true)

    document.dispatchEvent(new MouseEvent('click'))
    await nextTick()

    expect(wrapper.find('.project-dropdown').exists()).toBe(false)
  })

  it('should highlight current project in dropdown', async () => {
    const wrapper = mount(ProjectSelector, { props: defaultProps })
    await wrapper.find('.project-selector-btn').trigger('click')
    await nextTick()

    const currentItem = wrapper.find('.dropdown-item.current')
    expect(currentItem.exists()).toBe(true)
    expect(currentItem.text()).toContain('Project 1')
  })

  it('should show empty state when no recent projects', async () => {
    const wrapper = mount(ProjectSelector, {
      props: {
        ...defaultProps,
        recentProjects: [],
      },
    })
    await wrapper.find('.project-selector-btn').trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('workbench.noRecentProjects')
  })
})
