// altWinDirStat - fork-only menu commands (fork-owned; not part of upstream WinDirStat)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// at your option any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//

#include "pch.h"
#include "Ledger.h"
#include "ForkSettings.h"
#include "FileTreeControl.h"
#include "FileTabbedView.h"
#include "History.h"
#include "MessageBoxDlg.h"
#include "LedgerCompareDlg.h"
#include "ProgressDlg.h"

namespace
{
    std::wstring LedgerFilter()
    {
        return std::format(L"{} (*.ledger.csv)|*.ledger.csv|CSV (*.csv)|*.csv|{} (*.*)|*.*||",
            Localization::Lookup(IDS_LEDGER_FILE_FILTER), Localization::Lookup(IDS_ALL_FILES));
    }

    // Loads a ledger behind a progress dialog; reports errors itself.
    std::optional<Ledger::Snapshot> LoadWithProgress(const std::wstring& path)
    {
        std::optional<Ledger::Snapshot> snapshot;
        std::wstring error;
        CProgressDlg(0, CProgressDlg::Flags::NoCancel, GetMainWindow(), [&](CProgressDlg*)
        {
            snapshot = Ledger::Load(path, error);
        }).ShowModal();
        if (!snapshot) DisplayError(Localization::Format(IDS_LEDGER_INVALID, path, error));
        return snapshot;
    }
}

void CWinDirStatModel::OnUpdateFolderLedgerExport(CCmdUI* pCmdUI)
{
    pCmdUI->Enable(HasRootItem() && IsScanSettled());
}

void CWinDirStatModel::OnFolderLedgerExport()
{
    if (!HasRootItem() || !IsScanSettled()) return;
    const auto path = CDialog::PickFile(CDialog::FilePickerMode::Save, LedgerFilter());
    if (!path) return;

    bool saved = false;
    CProgressDlg(0, CProgressDlg::Flags::NoCancel, GetMainWindow(), [&](CProgressDlg*)
    {
        saved = Ledger::Save(*path, Ledger::FromScan(GetRootItem()));
    }).ShowModal();
    if (!saved) DisplayError(Localization::Format(IDS_LEDGER_SAVE_FAILED, *path));
}

void CWinDirStatModel::OnFolderLedgerCompare()
{
    const auto baselinePath = CDialog::PickFile(CDialog::FilePickerMode::Open, LedgerFilter());
    if (!baselinePath) return;
    auto baseline = LoadWithProgress(*baselinePath);
    if (!baseline) return;

    // Compare against the finished scan in the window, or else against a second ledger.
    std::optional<Ledger::Snapshot> current;
    if (HasRootItem() && IsScanSettled())
    {
        CProgressDlg(0, CProgressDlg::Flags::NoCancel, GetMainWindow(), [&](CProgressDlg*)
        {
            current = Ledger::FromScan(GetRootItem());
        }).ShowModal();
    }
    else
    {
        if (CMessageBoxDlg::Show(Localization::Lookup(IDS_LEDGER_PICK_CURRENT), MB_OKCANCEL | MB_ICONINFORMATION) != IDOK) return;
        const auto currentPath = CDialog::PickFile(CDialog::FilePickerMode::Open, LedgerFilter());
        if (!currentPath) return;
        current = LoadWithProgress(*currentPath);
        if (!current) return;
    }

    LedgerCompareDlg(GetMainWindow(), std::move(*baseline), std::move(*current)).ShowModal();
}

void CWinDirStatModel::OnForkTrackChanges()
{
    ForkSettings::TrackChanges = !ForkSettings::TrackChanges;
    if (!ForkSettings::TrackChanges) CMainFrame::Get()->GetFileTabbedView()->SetChangesTabVisibility(false);
}

void CWinDirStatModel::OnUpdateForkTrackChanges(CCmdUI* pCmdUI)
{
    pCmdUI->SetCheck(ForkSettings::TrackChanges ? 1 : 0);
}

void CWinDirStatModel::OnForkRelativeAges()
{
    ForkSettings::ShowRelativeAge = !ForkSettings::ShowRelativeAge;
    CFileTreeControl::Get()->Invalidate();
}

void CWinDirStatModel::OnUpdateForkRelativeAges(CCmdUI* pCmdUI)
{
    pCmdUI->SetCheck(ForkSettings::ShowRelativeAge ? 1 : 0);
}

void CWinDirStatModel::OnForkCleanHistory()
{
    const History::Usage usage = History::GetUsage();
    if (usage.snapshots == 0)
    {
        ShowMessageBox(Localization::Lookup(IDS_HISTORY_EMPTY), MB_OK | MB_ICONINFORMATION);
        return;
    }
    const std::wstring question = Localization::Format(IDS_HISTORY_CLEANUP_CONFIRM, FormatCount(usage.snapshots),
        FormatCount(usage.locations), FormatBytes(usage.bytes), ForkSettings::HistoryCapMB.Obj());
    if (ShowMessageBox(question, MB_YESNO | MB_ICONQUESTION) != IDYES) return;

    History::CleanUp();
    CMainFrame::Get()->GetFileTabbedView()->SetChangesTabVisibility(false);
}

namespace
{
    // DeepServer installs its main program next to this engine (deepserver-diskusage.exe).
    std::wstring DeepServerExe()
    {
        return (std::filesystem::path(GetAppFolder()) / L"DeepServer.exe").wstring();
    }

    // Quotes one command-line argument; a trailing backslash (drive roots) is doubled so it can't
    // escape the closing quote.
    std::wstring QuoteArgument(const std::wstring& value)
    {
        return L"\"" + value + (value.ends_with(L'\\') ? L"\\" : L"") + L"\"";
    }

    // Folders and drives in the selection that DeepServer can compare (one or two of them).
    std::vector<std::wstring> ComparableFolders(const std::vector<CItem*>& items)
    {
        std::vector<std::wstring> folders;
        for (const auto* item : items)
        {
            if (!item->IsTypeOrFlag(IT_DIRECTORY, IT_DRIVE)) return {};
            folders.push_back(item->GetPath());
        }
        if (folders.size() > 2) return {};
        return folders;
    }
}

void CWinDirStatModel::OnUpdateForkCompareInDeepServer(CCmdUI* pCmdUI)
{
    pCmdUI->Enable(!ComparableFolders(GetAllSelected()).empty() &&
        std::filesystem::exists(DeepServerExe()));
}

// Uses DeepServer's Explorer compare commands: with one folder it works like Explorer's
// "Compare with DeepServer" (the first pick is the left side, the next one opens the compare);
// with two folders it sets the left side, waits for that quick run, then opens the compare.
void CWinDirStatModel::OnForkCompareInDeepServer()
{
    const auto folders = ComparableFolders(GetAllSelected());
    const std::wstring exe = DeepServerExe();
    if (folders.empty() || !std::filesystem::exists(exe)) return;

    if (folders.size() == 2)
    {
        HANDLE process = nullptr;
        if (!ShellExecuteWrapper(exe, L"--shell-compare --select-left " + QuoteArgument(folders[0]),
            L"", GetMainWindowHandle(), L"", SW_NORMAL, 0, &process)) return;
        if (process != nullptr)
        {
            WaitForSingleObject(process, 10'000);
            CloseHandle(process);
        }
    }
    ShellExecuteWrapper(exe, L"--shell-compare " + QuoteArgument(folders.back()));
}
