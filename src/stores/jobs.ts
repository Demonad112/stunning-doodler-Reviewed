import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export type JobStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface JobProgress {
  current: number
  total: number | null
  message: string
}

export interface AppJob {
  id: string
  title: string
  status: JobStatus
  progress: JobProgress
  cancellable: boolean
}

/** Finished jobs kept for display; older ones are dropped so the list doesn't grow for the whole session. */
export const MAX_FINISHED_JOBS = 20

function isActive(job: AppJob): boolean {
  return job.status === 'queued' || job.status === 'running'
}

export const useJobsStore = defineStore('jobs', () => {
  const jobs = ref<AppJob[]>([])
  const runningJobs = computed(() => jobs.value.filter(isActive))

  /** Keeps every active job and the newest finished ones (the list is newest first). */
  function pruneFinished(): void {
    let finished = 0

    jobs.value = jobs.value.filter((job) => isActive(job) || ++finished <= MAX_FINISHED_JOBS)
  }

  function addJob(job: AppJob): void {
    const existingIndex = jobs.value.findIndex((item) => item.id === job.id)

    if (existingIndex >= 0) {
      jobs.value[existingIndex] = { ...job }
      pruneFinished()

      return
    }

    jobs.value.unshift({ ...job })
    pruneFinished()
  }

  function removeJob(id: string): void {
    jobs.value = jobs.value.filter((job) => job.id !== id)
  }

  function updateProgress(id: string, progress: JobProgress): void {
    const job = jobs.value.find((item) => item.id === id)

    if (!job) {
      return
    }

    job.progress = { ...progress }
  }

  function cancelJob(id: string): void {
    const job = jobs.value.find((item) => item.id === id)

    if (!job?.cancellable) {
      return
    }

    job.status = 'cancelled'
  }

  return {
    jobs,
    runningJobs,
    addJob,
    removeJob,
    updateProgress,
    cancelJob,
  }
})
