// altWinDirStat - plain-language tooltips on list column headers (fork-owned; not part of upstream WinDirStat)
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
#include "ForkTooltips.h"
#include "Item.h"
#include "ItemTop.h"

namespace
{
    constexpr UINT_PTR SubclassId = 0x41575453; // 'AWTS'

    struct HeaderTip
    {
        HWND list = nullptr;
        HWND tip = nullptr;
        ForkTooltips::Lookup lookup = nullptr;
        int column = -2;
        std::wstring text;
    };

    void UpdateTip(const HWND header, HeaderTip& state, const int column)
    {
        state.column = column;
        LVCOLUMNW col{ .mask = LVCF_SUBITEM };
        const bool known = column >= 0 && ::SendMessageW(state.list, LVM_GETCOLUMNW, column, reinterpret_cast<LPARAM>(&col));
        const std::wstring_view key = known ? state.lookup(col.iSubItem) : std::wstring_view{};
        state.text = key.empty() ? std::wstring() : Localization::Lookup(key);

        RECT rc{};
        if (column >= 0) ::SendMessageW(header, HDM_GETITEMRECT, column, reinterpret_cast<LPARAM>(&rc));
        TTTOOLINFOW ti{ .cbSize = sizeof(ti), .hwnd = header, .uId = 1, .rect = rc, .lpszText = state.text.data() };
        ::SendMessageW(state.tip, TTM_POP, 0, 0);
        ::SendMessageW(state.tip, TTM_NEWTOOLRECTW, 0, reinterpret_cast<LPARAM>(&ti));
        ::SendMessageW(state.tip, TTM_UPDATETIPTEXTW, 0, reinterpret_cast<LPARAM>(&ti));
        ::SendMessageW(state.tip, TTM_ACTIVATE, !state.text.empty(), 0);
    }

    LRESULT CALLBACK HeaderProc(const HWND hwnd, const UINT msg, const WPARAM wp, const LPARAM lp,
        const UINT_PTR id, const DWORD_PTR data)
    {
        auto* state = reinterpret_cast<HeaderTip*>(data);
        if (msg == WM_MOUSEMOVE)
        {
            HDHITTESTINFO hit{ .pt = { GET_X_LPARAM(lp), GET_Y_LPARAM(lp) } };
            const int column = static_cast<int>(::SendMessageW(hwnd, HDM_HITTEST, 0, reinterpret_cast<LPARAM>(&hit)));
            if (column != state->column) UpdateTip(hwnd, *state, column);
        }
        else if (msg == WM_NCDESTROY)
        {
            ::RemoveWindowSubclass(hwnd, HeaderProc, id);
            ::DestroyWindow(state->tip);
            delete state;
        }
        return ::DefSubclassProc(hwnd, msg, wp, lp);
    }
}

void ForkTooltips::AttachHeader(CListCtrl& list, const Lookup lookup)
{
    const HWND header = list.GetHeader().Handle();
    if (header == nullptr || lookup == nullptr) return;
    DWORD_PTR existing = 0;
    if (::GetWindowSubclass(header, HeaderProc, SubclassId, &existing)) return;

    const HWND tip = ::CreateWindowExW(WS_EX_TOPMOST, TOOLTIPS_CLASSW, nullptr, WS_POPUP | TTS_ALWAYSTIP | TTS_NOPREFIX,
        CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, header, nullptr, nullptr, nullptr);
    if (tip == nullptr) return;
    ::SendMessageW(tip, TTM_SETMAXTIPWIDTH, 0, ScaleForDpi(400));

    TTTOOLINFOW ti{ .cbSize = sizeof(ti), .uFlags = TTF_SUBCLASS, .hwnd = header, .uId = 1, .lpszText = const_cast<LPWSTR>(L"") };
    ::SendMessageW(tip, TTM_ADDTOOLW, 0, reinterpret_cast<LPARAM>(&ti));
    ::SendMessageW(tip, TTM_ACTIVATE, FALSE, 0);

    auto* state = new HeaderTip{ .list = list.Handle(), .tip = tip, .lookup = lookup };
    if (!::SetWindowSubclass(header, HeaderProc, SubclassId, reinterpret_cast<DWORD_PTR>(state)))
    {
        ::DestroyWindow(tip);
        delete state;
    }
}

std::wstring_view ForkTooltips::FileTreeTip(const int subitem)
{
    switch (subitem)
    {
    case COL_NAME:            return IDS_TIP_COL_NAME;
    case COL_SIZE_PROPORTION: return IDS_TIP_COL_SIZE_PROPORTION;
    case COL_PERCENTAGE:      return IDS_TIP_COL_PERCENTAGE;
    case COL_SIZE_PHYSICAL:   return IDS_TIP_COL_SIZE_PHYSICAL;
    case COL_SIZE_LOGICAL:    return IDS_TIP_COL_SIZE_LOGICAL;
    case COL_ITEMS:           return IDS_TIP_COL_ITEMS;
    case COL_FILES:           return IDS_TIP_COL_FILES;
    case COL_FOLDERS:         return IDS_TIP_COL_FOLDERS;
    case COL_LAST_CHANGE:     return IDS_TIP_COL_LAST_CHANGE;
    case COL_ATTRIBUTES:      return IDS_TIP_COL_ATTRIBUTES;
    case COL_OWNER:           return IDS_TIP_COL_OWNER;
    default:                  return {};
    }
}

std::wstring_view ForkTooltips::TopListTip(const int subitem)
{
    switch (subitem)
    {
    case COL_ITEMTOP_NAME:          return IDS_TIP_COL_NAME;
    case COL_ITEMTOP_SIZE_PHYSICAL: return IDS_TIP_COL_SIZE_PHYSICAL;
    case COL_ITEMTOP_SIZE_LOGICAL:  return IDS_TIP_COL_SIZE_LOGICAL;
    case COL_ITEMTOP_LAST_CHANGE:   return IDS_TIP_COL_LAST_CHANGE;
    default:                        return {};
    }
}

std::wstring_view ForkTooltips::ChangesTip(const int subitem)
{
    // Column order matches CFileChangesView's Column enum.
    static constexpr std::array tips{ IDS_TIP_CHG_FOLDER, IDS_TIP_CHG_CHANGE, IDS_TIP_CHG_BEFORE,
        IDS_TIP_CHG_NOW, IDS_TIP_CHG_DIFF, IDS_TIP_CHG_FILES };
    return subitem >= 0 && subitem < static_cast<int>(tips.size()) ? tips[subitem] : std::wstring_view{};
}
