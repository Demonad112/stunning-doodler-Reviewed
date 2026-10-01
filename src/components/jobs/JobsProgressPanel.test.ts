import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import JobsProgressPanel from './JobsProgressPanel.vue'

describe('JobsProgressPanel', () => {
  it('renders running jobs with progress and cancel controls', async () => {
    const wrapper = mount(JobsProgressPanel, {
      props: {
        jobs: [
          {
            id: 'scan-1',
            title: 'Scan folder',
            status: 'running',
            progress: { current: 5, total: 10, message: 'Scanning' },
            cancellable: true,
          },
        ],
      },
    })

    expect(wrapper.text()).toContain('Scan folder')
    expect(wrapper.text()).toContain('50%')
    expect(wrapper.text()).toContain('Scanning')

    await wrapper.find('[data-testid="cancel-job-scan-1"]').trigger('click')

    expect(wrapper.emitted('cancel')).toEqual([['scan-1']])
  })

  it('renders an empty state when no jobs are running', () => {
    const wrapper = mount(JobsProgressPanel, {
      props: {
        jobs: [],
      },
    })

    expect(wrapper.text()).toContain('No running jobs')
  })

  it('does not pass the Cancel click on to a click handler around the panel', async () => {
    const onPanelClick = vi.fn()
    const onCancel = vi.fn()
    const Host = defineComponent({
      setup() {
        return (): ReturnType<typeof h> =>
          h(JobsProgressPanel, {
            jobs: [
              {
                id: 'copy-1',
                title: 'Copy',
                status: 'running',
                progress: { current: 1, total: 2, message: '' },
                cancellable: true,
              },
            ],
            onClick: onPanelClick,
            onCancel,
          })
      },
    })
    const wrapper = mount(Host)

    await wrapper.find('[data-testid="cancel-job-copy-1"]').trigger('click')

    expect(onCancel).toHaveBeenCalledWith('copy-1')
    expect(onPanelClick).not.toHaveBeenCalled()
  })
})
