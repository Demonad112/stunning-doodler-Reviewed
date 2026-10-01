import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { MAX_FINISHED_JOBS, useJobsStore } from './jobs'

describe('useJobsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('adds jobs and exposes running jobs', () => {
    const store = useJobsStore()

    store.addJob({
      id: 'scan-1',
      title: 'Scan folder',
      status: 'running',
      progress: { current: 2, total: 10, message: 'Scanning' },
      cancellable: true,
    })

    expect(store.runningJobs).toHaveLength(1)
    expect(store.runningJobs[0]?.title).toBe('Scan folder')
  })

  it('updates progress and cancels cancellable jobs', () => {
    const store = useJobsStore()

    store.addJob({
      id: 'scan-1',
      title: 'Scan folder',
      status: 'running',
      progress: { current: 2, total: 10, message: 'Scanning' },
      cancellable: true,
    })
    store.updateProgress('scan-1', { current: 7, total: 10, message: 'Indexing' })
    store.cancelJob('scan-1')

    expect(store.jobs[0]?.progress.current).toBe(7)
    expect(store.jobs[0]?.status).toBe('cancelled')
  })

  it('keeps running jobs and only the newest finished ones', () => {
    const store = useJobsStore()

    store.addJob({
      id: 'running',
      title: 'Still running',
      status: 'running',
      progress: { current: 0, total: null, message: '' },
      cancellable: true,
    })
    for (let index = 0; index < MAX_FINISHED_JOBS + 5; index += 1) {
      store.addJob({
        id: `done-${String(index)}`,
        title: 'Done',
        status: 'completed',
        progress: { current: 1, total: 1, message: '' },
        cancellable: false,
      })
    }

    expect(store.jobs).toHaveLength(MAX_FINISHED_JOBS + 1)
    expect(store.jobs.some((job) => job.id === 'running')).toBe(true)
    expect(store.jobs.some((job) => job.id === 'done-0')).toBe(false)
    expect(store.jobs[0]?.id).toBe(`done-${String(MAX_FINISHED_JOBS + 4)}`)

    store.removeJob('running')

    expect(store.runningJobs).toHaveLength(0)
  })
})
