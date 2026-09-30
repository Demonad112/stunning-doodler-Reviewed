import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import DiskUsageView from './DiskUsageView.vue'
import {
  compareSnapshots,
  listDrives,
  listSnapshots,
  openDiskUsage,
  takeSnapshot,
  type Comparison,
  type SnapshotInfo,
} from '@/api/diskusage'
import { diskUsageLocationsStorageKey } from '@/app/diskUsage'
import { pickNativePath } from '@/app/filePicker'
import { useSessionLaunchStore } from '@/stores/sessionLaunch'

const push = vi.fn()
const routeQuery: { path?: string } = {}

vi.mock('vue-router', () => ({
  useRouter: () => ({ push }),
  useRoute: () => ({ query: routeQuery, path: '/disk/usage' }),
}))

vi.mock('@/api/diskusage', () => ({
  listDrives: vi.fn(),
  listSnapshots: vi.fn(),
  takeSnapshot: vi.fn(),
  compareSnapshots: vi.fn(),
  openDiskUsage: vi.fn(),
}))

vi.mock('@/app/filePicker', () => ({
  pickNativePath: vi.fn(),
}))

function snapshot(name: string, takenAtUtc: string): SnapshotInfo {
  return { path: `H\\${name}`, fileName: name, takenAtUtc, sizeBytes: 100 }
}

const newer = snapshot('20260929-100000-000.ledger.csv', '2026-09-29 10:00:00')
const older = snapshot('20260928-100000-000.ledger.csv', '2026-09-28 10:00:00')

const comparison: Comparison = {
  rows: [
    {
      folder: 'small',
      change: 'grown',
      baselineSize: 1024,
      currentSize: 2 * 1024 ** 2,
      sizeChange: 2 * 1024 ** 2 - 1024,
      baselineFiles: 1,
      currentFiles: 2,
      filesChange: 1,
    },
    {
      folder: 'big',
      change: 'added',
      baselineSize: null,
      currentSize: 5 * 1024 ** 3,
      sizeChange: 5 * 1024 ** 3,
      baselineFiles: null,
      currentFiles: 10,
      filesChange: 10,
    },
  ],
  warning: null,
}

function mountView(): ReturnType<typeof mount> {
  return mount(DiskUsageView)
}

describe('DiskUsageView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    setActivePinia(createPinia())
    delete routeQuery.path
    vi.mocked(listDrives).mockResolvedValue([
      { root: 'C:\\', label: 'System', kind: 'fixed', totalBytes: 1024 ** 4, freeBytes: 1024 ** 3 },
    ])
    vi.mocked(listSnapshots).mockResolvedValue([newer, older])
    vi.mocked(compareSnapshots).mockResolvedValue(comparison)
    vi.mocked(takeSnapshot).mockReset()
    vi.mocked(openDiskUsage).mockReset()
    vi.mocked(pickNativePath).mockReset()
  })

  it('offers drives and asks for a location when nothing was used before', async () => {
    const wrapper = mountView()

    await flushPromises()

    const options = wrapper.findAll('#disk-usage-location-choices option')

    expect(options.map((option) => option.attributes('value'))).toEqual(['C:\\'])
    expect(wrapper.find('[data-testid="disk-usage-changes"]').exists()).toBe(false)
    expect(listSnapshots).not.toHaveBeenCalled()
  })

  it('compares the two newest snapshots of the chosen location, largest change first', async () => {
    const wrapper = mountView()

    await flushPromises()

    const input = wrapper.find('[data-testid="disk-usage-location"]')

    await input.setValue('C:\\')
    await input.trigger('change')
    await flushPromises()

    expect(listSnapshots).toHaveBeenCalledWith('C:\\')
    expect(compareSnapshots).toHaveBeenCalledWith(older.path, newer.path, false)
    const rows = wrapper.findAll('[data-testid="disk-usage-change-row"]')

    expect(rows.map((row) => row.find('td').text())).toEqual(['big', 'small'])
    expect(wrapper.find('[data-testid="disk-usage-drive"]').text()).toContain('System')
    expect(JSON.parse(localStorage.getItem(diskUsageLocationsStorageKey) ?? '[]')).toEqual(['C:\\'])
  })

  it('shows the engine warning when the filters changed between snapshots', async () => {
    vi.mocked(compareSnapshots).mockResolvedValue({ ...comparison, warning: 'Filters changed.' })
    routeQuery.path = 'D:\\Data'

    const wrapper = mountView()

    await flushPromises()

    expect(listSnapshots).toHaveBeenCalledWith('D:\\Data')
    expect(wrapper.find('[data-testid="disk-usage-warning"]').text()).toBe('Filters changed.')
  })

  it('re-compares with every change when Show all is ticked', async () => {
    routeQuery.path = 'D:\\Data'
    const wrapper = mountView()

    await flushPromises()

    await wrapper.find('[data-testid="disk-usage-show-all"]').setValue(true)
    await flushPromises()

    expect(compareSnapshots).toHaveBeenLastCalledWith(older.path, newer.path, true)
  })

  it('explains that two snapshots are needed, and takes one on request', async () => {
    vi.mocked(listSnapshots).mockResolvedValueOnce([]).mockResolvedValueOnce([newer])
    vi.mocked(takeSnapshot).mockResolvedValue(newer)
    routeQuery.path = 'D:\\Data'

    const wrapper = mountView()

    await flushPromises()
    expect(wrapper.find('[data-testid="disk-usage-need-snapshots"]').exists()).toBe(true)

    await wrapper.find('[data-testid="disk-usage-take-snapshot"]').trigger('click')
    await flushPromises()

    expect(takeSnapshot).toHaveBeenCalledWith('D:\\Data')
    expect(listSnapshots).toHaveBeenCalledTimes(2)
    expect(compareSnapshots).not.toHaveBeenCalled()
  })

  it('shows a failed snapshot as an error', async () => {
    vi.mocked(takeSnapshot).mockRejectedValue({ message: 'disk-usage engine timed out' })
    routeQuery.path = 'D:\\Data'

    const wrapper = mountView()

    await flushPromises()
    await wrapper.find('[data-testid="disk-usage-take-snapshot"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="disk-usage-error"]').text()).toBe(
      'disk-usage engine timed out',
    )
  })

  it('browses for a folder and opens the treemap on it', async () => {
    vi.mocked(pickNativePath).mockResolvedValue('E:\\Media')
    const wrapper = mountView()

    await flushPromises()

    await wrapper.find('[data-testid="disk-usage-browse"]').trigger('click')
    await flushPromises()
    await wrapper.find('[data-testid="disk-usage-open-treemap"]').trigger('click')

    expect(pickNativePath).toHaveBeenCalledWith({ directory: true })
    expect(listSnapshots).toHaveBeenCalledWith('E:\\Media')
    expect(openDiskUsage).toHaveBeenCalledWith('E:\\Media')
  })

  it('opens Folder Compare with a changed folder on the left', async () => {
    routeQuery.path = 'D:\\Data\\'
    const wrapper = mountView()

    await flushPromises()

    await wrapper.find('[data-testid="disk-usage-compare-folder"]').trigger('click')

    const launch = useSessionLaunchStore().pendingLaunch

    expect(launch?.route).toBe('/compare/folder')
    expect(launch?.locations.left?.uri).toBe('D:\\Data\\big')
    expect(push).toHaveBeenCalled()
  })
})
