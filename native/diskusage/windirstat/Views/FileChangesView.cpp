// altWinDirStat - "Changes" tab (fork-owned; not part of upstream WinDirStat)
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
#include "FileChangesView.h"
#include "FileTabbedView.h"
#include "ForkFormat.h"
#include "ForkTooltips.h"

namespace
{
    enum Column : int { ColFolder, ColChange, ColBefore, ColNow, ColDelta, ColFiles, ColCount };

    LONGLONG Magnitude(const LONGLONG v) { return v < 0 ? -v : v; }

    LONGLONG SortValue(const Ledger::DiffRow& row, const int column)
    {
        switch (column)
        {
        case ColChange: return std::to_underlying(row.change);
        case ColBefore: return row.before ? static_cast<LONGLONG>(row.before->size) : -1;
        case ColNow:    return row.after ? static_cast<LONGLONG>(row.after->size) : -1;
        case ColDelta:  return Magnitude(row.sizeDelta);   // Biggest change first, grown or shrunk
        case ColFiles:  return row.filesDelta;
        default:        return 0;
        }
    }

    // Finds the live item for a folder row by walking relative-path components from the scan root.
    // Exact name first, so "Data" and "data" in a case-sensitive folder resolve to the right one.
    CItem* FindLiveFolder(const Ledger::DiffRow& row)
    {
        CItem* item = CWinDirStatModel::Get()->GetRootItem();
        if (item == nullptr || row.after == nullptr) return nullptr;
        if (row.relative == L".") return item;
        for (const auto& part : SplitString(row.relative, wds::chrBackslash))
        {
            if (item->IsLeaf()) return nullptr;
            const auto& children = item->GetChildren();
            auto it = std::ranges::find_if(children, [&](const CItem* child) { return child->GetNameView() == part; });
            if (it == children.end()) it = std::ranges::find_if(children, [&](const CItem* child)
            {
                const std::wstring_view name = child->GetNameView();
                return name.size() == part.size() && _wcsnicmp(name.data(), part.c_str(), part.size()) == 0;
            });
            if (it == children.end()) return nullptr;
            item = *it;
        }
        return item;
    }
}

int CFileChangesView::OnCreate(const LPCREATESTRUCT lpCreateStruct)
{
    if (CWinDirStatPane::OnCreate(lpCreateStruct) == -1) return -1;

    const CRect rect(0, 0, 0, 0);
    m_summary.Create(L"", WS_CHILD | WS_VISIBLE | SS_LEFT | SS_ENDELLIPSIS, rect, this, IDC_CHANGES_SUMMARY);
    m_showAll.Create(Localization::Lookup(IDS_CHANGES_SHOW_ALL).c_str(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX, rect, this, IDC_CHANGES_SHOW_ALL);
    m_list.Create(WS_CHILD | WS_VISIBLE | WS_TABSTOP | LVS_REPORT | LVS_OWNERDATA | LVS_SHOWSELALWAYS | LVS_SINGLESEL,
        rect, this, IDC_CHANGES_LIST);
    m_list.SetExtendedStyle(LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER | LVS_EX_LABELTIP | LVS_EX_GRIDLINES);

    const std::array columns{ IDS_LEDGER_COL_FOLDER, IDS_LEDGER_COL_CHANGE, IDS_CHANGES_COL_BEFORE,
        IDS_CHANGES_COL_NOW, IDS_CHANGES_COL_DIFF, IDS_CHANGES_COL_FILES };
    const std::array widths{ 360, 90, 90, 90, 100, 80 };
    for (int i = 0; i < ColCount; ++i)
        m_list.InsertColumn(i, Localization::Lookup(columns[i]), i <= ColChange ? LVCFMT_LEFT : LVCFMT_RIGHT, ScaleForDpi(widths[i]));
    ForkTooltips::AttachHeader(m_list, ForkTooltips::ChangesTip);
    m_list.SetBkColor(DarkMode::SystemColor(COLOR_WINDOW));
    m_list.SetTextBkColor(DarkMode::SystemColor(COLOR_WINDOW));
    m_list.SetTextColor(DarkMode::SystemColor(COLOR_WINDOWTEXT));

    DarkMode::AdjustControls(m_hWnd);
    OnFontSizeChanged(0, 0);
    return 0;
}

void CFileChangesView::OnFontSizeChanged(int, int)
{
    m_font.Create(-ScaleForDpi(12), FW_NORMAL, wds::strFontSegoeUI);
    m_summary.SetFont(m_font);
    m_showAll.SetFont(m_font);
    const CRect rc = GetClientRect();
    OnSize(SIZE_RESTORED, rc.Width(), rc.Height());
}

void CFileChangesView::OnSize(UINT, const int cx, const int cy)
{
    if (m_list.Handle() == nullptr) return;
    const int margin = ScaleForDpi(6), rowH = ScaleForDpi(22), checkW = ScaleForDpi(200);
    m_summary.MoveWindow(margin, margin, std::max(0, cx - checkW - 3 * margin), rowH);
    m_showAll.MoveWindow(std::max(margin, cx - checkW - margin), margin, checkW, rowH);
    const int top = rowH + 2 * margin;
    m_list.MoveWindow(0, top, cx, std::max(0, cy - top));
    Invalidate();
}

void CFileChangesView::OnDraw(CDC* pDC)
{
    pDC->FillSolidRect(GetClientRect(), DarkMode::SystemColor(COLOR_3DFACE));
}

HBRUSH CFileChangesView::OnCtlColor(CDC* pDC, CWnd*, const UINT nCtlColor)
{
    if (nCtlColor != CTLCOLOR_STATIC && nCtlColor != CTLCOLOR_BTN) return nullptr;
    pDC->SetTextColor(DarkMode::SystemColor(COLOR_WINDOWTEXT));
    pDC->SetBkColor(DarkMode::SystemColor(COLOR_3DFACE));
    if (m_background != nullptr) DeleteObject(m_background);
    m_background = CreateSolidBrush(DarkMode::SystemColor(COLOR_3DFACE));
    return m_background;
}

void CFileChangesView::SetResult(std::shared_ptr<const History::Result> result)
{
    m_result = std::move(result);
    UpdateSummary();
    UpdateList();
}

void CFileChangesView::UpdateSummary()
{
    if (!m_result)
    {
        m_summary.SetText(L"");
        return;
    }
    std::array<size_t, 6> counts{};
    for (const size_t i : Ledger::Significant(m_result->rows, Ledger::DefaultMinOwnDelta, false))
        ++counts[std::to_underlying(m_result->rows[i].change)];
    const auto count = [&](const Ledger::Change c) { return FormatCount(counts[std::to_underlying(c)]); };

    m_summary.SetText(Localization::Format(IDS_CHANGES_SUMMARY,
        FormatFileTime(m_result->baselineTime), ForkFormat::RelativeAge(m_result->baselineTime, ForkFormat::Now()),
        Ledger::SignedBytes(m_result->summary.sizeDelta),
        count(Ledger::Change::Added), count(Ledger::Change::Removed),
        count(Ledger::Change::Grown), count(Ledger::Change::Shrunk)));
}

void CFileChangesView::UpdateList()
{
    m_visible.clear();
    if (m_result)
    {
        const bool all = IsDlgButtonChecked(m_hWnd, IDC_CHANGES_SHOW_ALL) == BST_CHECKED;
        m_visible = Ledger::Significant(m_result->rows, Ledger::DefaultMinOwnDelta, all);
        std::ranges::stable_sort(m_visible, [this](const size_t a, const size_t b)
        {
            const auto& left = m_result->rows[m_descending ? b : a];
            const auto& right = m_result->rows[m_descending ? a : b];
            if (m_sortColumn == ColFolder) return _wcsicmp(left.relative.c_str(), right.relative.c_str()) < 0;
            return SortValue(left, m_sortColumn) < SortValue(right, m_sortColumn);
        });
    }
    m_list.SetItemCountEx(static_cast<int>(m_visible.size()), LVSICF_NOSCROLL);
    m_list.Invalidate();
}

std::wstring CFileChangesView::CellText(const Ledger::DiffRow& row, const int column) const
{
    switch (column)
    {
    case ColFolder: return row.relative;
    case ColChange: return Ledger::ChangeName(row.change);
    case ColBefore: return row.before ? FormatBytes(row.before->size) : std::wstring();
    case ColNow:    return row.after ? FormatBytes(row.after->size) : std::wstring();
    case ColDelta:  return Ledger::SignedBytes(row.sizeDelta);
    case ColFiles:  return Ledger::SignedCount(row.filesDelta);
    default:        return {};
    }
}

void CFileChangesView::OnShowAll()
{
    UpdateList();
}

void CFileChangesView::OnDisplayInfo(NMHDR* header, LRESULT* result)
{
    *result = 0;
    auto& item = reinterpret_cast<NMLVDISPINFO*>(header)->item;
    if (!m_result || !(item.mask & LVIF_TEXT) || item.iItem < 0 || static_cast<size_t>(item.iItem) >= m_visible.size()) return;
    m_cell = CellText(m_result->rows[m_visible[item.iItem]], item.iSubItem);
    item.pszText = m_cell.data();
}

void CFileChangesView::OnColumnClick(NMHDR* header, LRESULT* result)
{
    *result = 0;
    const int column = reinterpret_cast<NMLISTVIEW*>(header)->iSubItem;
    if (column < 0 || column >= ColCount) return;
    m_descending = column == m_sortColumn ? !m_descending : column != ColFolder;
    m_sortColumn = column;
    UpdateList();
}

void CFileChangesView::OnCustomDraw(NMHDR* header, LRESULT* result)
{
    *result = CDRF_DODEFAULT;
    auto* draw = reinterpret_cast<NMLVCUSTOMDRAW*>(header);
    if (draw->nmcd.dwDrawStage == CDDS_PREPAINT) *result = CDRF_NOTIFYITEMDRAW;
    else if (draw->nmcd.dwDrawStage == CDDS_ITEMPREPAINT && m_result && draw->nmcd.dwItemSpec < m_visible.size())
    {
        draw->clrText = Ledger::ChangeColor(m_result->rows[m_visible[draw->nmcd.dwItemSpec]].change);
        if (DarkMode::IsDarkModeActive()) draw->clrTextBk = DarkMode::SystemColor(COLOR_WINDOW);
    }
}

void CFileChangesView::OnDoubleClick(NMHDR* header, LRESULT* result)
{
    *result = 0;
    const int index = reinterpret_cast<NMITEMACTIVATE*>(header)->iItem;
    if (!m_result || index < 0 || static_cast<size_t>(index) >= m_visible.size()) return;

    // Reveal a folder that still exists in the All Files tree; removed folders have nothing to show.
    if (CItem* item = FindLiveFolder(m_result->rows[m_visible[index]]); item != nullptr)
    {
        CMainFrame::Get()->GetFileTabbedView()->SetActiveFileTreeView();
        CWinDirStatModel::Get()->NotifyPanes(MODEL_CHANGE_SELECTION_ACTION, item);
    }
}
