// altWinDirStat - "Changes" tab: folders that changed since the previous session's scan
// (fork-owned; not part of upstream WinDirStat)
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

#pragma once

#include "pch.h"
#include "WinDirStatPane.h"
#include "History.h"
#include "LedgerListCtrl.h"
#include "ForkResource.h"

class CFileChangesView final : public MessageTarget<CFileChangesView, CWinDirStatPane>
{
public:
    // Shows a new comparison (nullptr clears the view).
    void SetResult(std::shared_ptr<const History::Result> result);
    bool HasResult() const { return m_result != nullptr; }

    void OnDraw(CDC* pDC) override;
    void OnFontSizeChanged(int, int) override;
    static std::span<const RouteEntry> Routes();

protected:
    int OnCreate(LPCREATESTRUCT lpCreateStruct);
    void OnSize(UINT nType, int cx, int cy);
    bool OnEraseBkgnd(CDC*) { return true; }
    HBRUSH OnCtlColor(CDC* pDC, CWnd* pWnd, UINT nCtlColor);
    void OnShowAll();
    void OnDisplayInfo(NMHDR* header, LRESULT* result);
    void OnColumnClick(NMHDR* header, LRESULT* result);
    void OnCustomDraw(NMHDR* header, LRESULT* result);
    void OnDoubleClick(NMHDR* header, LRESULT* result);

private:
    void UpdateList();
    void UpdateSummary();
    std::wstring CellText(const Ledger::DiffRow& row, int column) const;

    CStatic m_summary;
    CButton m_showAll;
    CLedgerListCtrl m_list;
    CFont m_font;
    HBRUSH m_background = nullptr;
    std::shared_ptr<const History::Result> m_result;
    std::vector<size_t> m_visible;
    std::wstring m_cell;
    int m_sortColumn = 4;       // Difference
    bool m_descending = true;
};

inline std::span<const RouteEntry> CFileChangesView::Routes()
{
    static constexpr std::array entries
    {
        Route::Window<&OnCreate>(WM_CREATE),
        Route::Window<&OnSize>(WM_SIZE),
        Route::Window<&OnEraseBkgnd>(WM_ERASEBKGND),
        Route::Window<&OnCtlColor>(WM_CTLCOLOR),
        Route::Control<&OnShowAll>(BN_CLICKED, IDC_CHANGES_SHOW_ALL),
        Route::Notify<&OnDisplayInfo>(LVN_GETDISPINFO, IDC_CHANGES_LIST),
        Route::Notify<&OnColumnClick>(LVN_COLUMNCLICK, IDC_CHANGES_LIST),
        Route::Notify<&OnCustomDraw>(NM_CUSTOMDRAW, IDC_CHANGES_LIST),
        Route::Notify<&OnDoubleClick>(NM_DBLCLK, IDC_CHANGES_LIST),
    };
    return entries;
}
