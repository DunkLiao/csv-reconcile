import React, { useState, useMemo } from 'react';
import {
  Search,
  Download,
  RotateCcw,
  ChevronLeft,
  ChevronRight,
  Columns,
  AlertCircle,
  FileSpreadsheet,
} from 'lucide-react';
import { CompareOptions, CompareResult, Difference, DifferenceType, KeyValue } from '../types';
import { exportExcel, saveExcelDialog } from '../lib/tauri';

interface DifferenceTableProps {
  result: CompareResult;
  options: CompareOptions;
  onReset: () => void;
}

export const DifferenceTable: React.FC<DifferenceTableProps> = ({
  result,
  options,
  onReset,
}) => {
  const [activeTab, setActiveTab] = useState<'diffs' | 'columns' | 'duplicates'>('diffs');
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<string>('ALL');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(50);
  const [exporting, setExporting] = useState(false);
  const [exportSuccess, setExportSuccess] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);

  // Filter differences
  const filteredDiffs = useMemo(() => {
    return result.differences.filter((d) => {
      // Type filter
      if (typeFilter !== 'ALL') {
        if (typeFilter === 'VALUE_CHANGED' && d.difference_type !== 'VALUE_CHANGED') return false;
        if (typeFilter === 'A_ONLY' && d.difference_type !== 'A_ONLY') return false;
        if (typeFilter === 'B_ONLY' && d.difference_type !== 'B_ONLY') return false;
        if (typeFilter === 'DUPLICATE' && !d.difference_type.startsWith('DUPLICATE_KEY')) return false;
        if (typeFilter === 'KEY_ERROR' && (!d.difference_type.startsWith('EMPTY_KEY') && !d.difference_type.startsWith('INCOMPLETE_KEY'))) return false;
      }

      // Search filter
      if (search.trim()) {
        const q = search.toLowerCase();
        const keyMatch = d.key_values.some(
          (kv) => kv.value.toLowerCase().includes(q) || kv.column.toLowerCase().includes(q)
        );
        const colMatch = d.column_name?.toLowerCase().includes(q) ?? false;
        const valAMatch = d.value_a?.toLowerCase().includes(q) ?? false;
        const valBMatch = d.value_b?.toLowerCase().includes(q) ?? false;
        const rowAMatch = d.row_a?.toString().includes(q) ?? false;
        const rowBMatch = d.row_b?.toString().includes(q) ?? false;
        const typeMatch = d.difference_type.toLowerCase().includes(q);

        return keyMatch || colMatch || valAMatch || valBMatch || rowAMatch || rowBMatch || typeMatch;
      }

      return true;
    });
  }, [result.differences, typeFilter, search]);

  const totalPages = Math.max(1, Math.ceil(filteredDiffs.length / pageSize));
  const currentPageDiffs = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredDiffs.slice(start, start + pageSize);
  }, [filteredDiffs, page, pageSize]);

  const handleExport = async () => {
    try {
      setExporting(true);
      setExportError(null);
      setExportSuccess(null);

      const path = await saveExcelDialog();
      if (!path) {
        setExporting(false);
        return;
      }

      await exportExcel(path, options, result);
      setExportSuccess(`成功匯出 Excel：${path}`);
    } catch (err: any) {
      setExportError(err?.message || '匯出失敗，請確認檔案是否被其他程式開啟或磁碟權限。');
    } finally {
      setExporting(false);
    }
  };

  const getBadgeStyle = (type: DifferenceType) => {
    switch (type) {
      case 'VALUE_CHANGED':
        return 'bg-rose-50 text-rose-700 border-rose-200';
      case 'A_ONLY':
        return 'bg-amber-50 text-amber-700 border-amber-200';
      case 'B_ONLY':
        return 'bg-sky-50 text-sky-700 border-sky-200';
      case 'DUPLICATE_KEY_A':
      case 'DUPLICATE_KEY_B':
        return 'bg-purple-50 text-purple-700 border-purple-200';
      case 'EMPTY_KEY_A':
      case 'EMPTY_KEY_B':
      case 'INCOMPLETE_KEY_A':
      case 'INCOMPLETE_KEY_B':
        return 'bg-orange-50 text-orange-700 border-orange-200';
      default:
        return 'bg-slate-100 text-slate-700 border-slate-200';
    }
  };

  return (
    <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-6 flex flex-col gap-4">
      {/* Top Action Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 pb-3 border-b border-slate-100">
        <div className="flex items-center gap-2">
          {/* Navigation Tabs */}
          <button
            type="button"
            onClick={() => setActiveTab('diffs')}
            className={`px-3 py-1.5 rounded-lg text-xs font-bold transition flex items-center gap-1.5 ${
              activeTab === 'diffs'
                ? 'bg-blue-600 text-white'
                : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
            }`}
          >
            <FileSpreadsheet className="w-3.5 h-3.5" />
            差異明細清單 ({result.differences.length.toLocaleString()})
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('columns')}
            className={`px-3 py-1.5 rounded-lg text-xs font-bold transition flex items-center gap-1.5 ${
              activeTab === 'columns'
                ? 'bg-blue-600 text-white'
                : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
            }`}
          >
            <Columns className="w-3.5 h-3.5" />
            欄位結構比對 ({result.column_differences.length})
          </button>
          {result.duplicate_key_records.length > 0 && (
            <button
              type="button"
              onClick={() => setActiveTab('duplicates')}
              className={`px-3 py-1.5 rounded-lg text-xs font-bold transition flex items-center gap-1.5 ${
                activeTab === 'duplicates'
                  ? 'bg-purple-600 text-white'
                  : 'bg-purple-50 text-purple-700 hover:bg-purple-100'
              }`}
            >
              <AlertCircle className="w-3.5 h-3.5" />
              重複 Key 清單 ({result.duplicate_key_records.length})
            </button>
          )}
        </div>

        {/* Right buttons: Reset & Export */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={onReset}
            className="inline-flex items-center gap-1.5 px-3.5 py-2 text-xs font-semibold bg-slate-100 hover:bg-slate-200 border border-slate-300 rounded-lg text-slate-700 transition"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            重新設定
          </button>

          <button
            type="button"
            disabled={exporting}
            onClick={handleExport}
            className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-bold bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg shadow-sm transition active:scale-98 disabled:opacity-50"
          >
            <Download className="w-3.5 h-3.5" />
            {exporting ? '正在匯出...' : '匯出 Excel (.xlsx)'}
          </button>
        </div>
      </div>

      {/* Export Notifications */}
      {exportSuccess && (
        <div className="p-3 bg-emerald-50 text-emerald-800 border border-emerald-200 rounded-lg text-xs font-medium">
          {exportSuccess}
        </div>
      )}
      {exportError && (
        <div className="p-3 bg-rose-50 text-rose-800 border border-rose-200 rounded-lg text-xs font-medium">
          {exportError}
        </div>
      )}

      {/* Tab: Differences Table */}
      {activeTab === 'diffs' && (
        <div className="flex flex-col gap-3">
          {/* Search and Filters */}
          <div className="flex flex-wrap items-center justify-between gap-3 bg-slate-50 p-2.5 rounded-lg border border-slate-200">
            {/* Search Input */}
            <div className="relative flex-1 min-w-[200px] max-w-sm">
              <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
              <input
                type="text"
                value={search}
                onChange={(e) => {
                  setSearch(e.target.value);
                  setPage(1);
                }}
                placeholder="搜尋 Key、欄位名稱、資料值或列號..."
                className="w-full pl-9 pr-3 py-1.5 text-xs bg-white border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Type Filters */}
            <div className="flex items-center gap-1.5 text-xs">
              <span className="text-slate-500 text-[11px] font-medium mr-1">篩選類型：</span>
              {[
                { id: 'ALL', label: '全部' },
                { id: 'VALUE_CHANGED', label: '內容變更' },
                { id: 'A_ONLY', label: 'A 獨有' },
                { id: 'B_ONLY', label: 'B 獨有' },
                { id: 'DUPLICATE', label: '重複 Key' },
                { id: 'KEY_ERROR', label: '空 Key 異常' },
              ].map((f) => (
                <button
                  key={f.id}
                  type="button"
                  onClick={() => {
                    setTypeFilter(f.id);
                    setPage(1);
                  }}
                  className={`px-2.5 py-1 rounded text-xs font-medium transition ${
                    typeFilter === f.id
                      ? 'bg-blue-600 text-white font-semibold'
                      : 'bg-white text-slate-600 hover:bg-slate-200 border border-slate-200'
                  }`}
                >
                  {f.label}
                </button>
              ))}
            </div>
          </div>

          {/* Table */}
          <div className="overflow-x-auto border border-slate-200 rounded-lg">
            <table className="w-full text-xs text-left">
              <thead className="bg-slate-100 text-slate-700 font-bold border-b border-slate-200 uppercase tracking-wider">
                <tr>
                  {result.key_columns.map((k) => (
                    <th key={k} className="py-2.5 px-3 font-semibold text-blue-900 bg-blue-50/50">
                      {k} (Key)
                    </th>
                  ))}
                  <th className="py-2.5 px-3">差異欄位</th>
                  <th className="py-2.5 px-3">File A 內容</th>
                  <th className="py-2.5 px-3">File B 內容</th>
                  <th className="py-2.5 px-3 text-right">Row A</th>
                  <th className="py-2.5 px-3 text-right">Row B</th>
                  <th className="py-2.5 px-3 text-center">差異類型</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 font-mono">
                {currentPageDiffs.map((d: Difference, idx: number) => {
                  const keyMap = new Map<string, string>(d.key_values.map((kv: KeyValue) => [kv.column, kv.value]));
                  return (
                    <tr key={idx} className="hover:bg-slate-50 transition">
                      {/* Key columns */}
                      {result.key_columns.map((k: string) => (
                        <td key={k} className="py-2 px-3 text-slate-800 font-medium whitespace-nowrap bg-blue-50/20">
                          {String(keyMap.get(k) ?? '-')}
                        </td>
                      ))}
                      {/* Column */}
                      <td className="py-2 px-3 font-sans font-semibold text-slate-800">
                        {d.column_name || '-'}
                      </td>
                      {/* File A Value */}
                      <td className="py-2 px-3 text-rose-700 bg-rose-50/30 font-medium max-w-xs truncate" title={d.value_a}>
                        {d.value_a !== undefined && d.value_a !== null ? d.value_a : '-'}
                      </td>
                      {/* File B Value */}
                      <td className="py-2 px-3 text-sky-700 bg-sky-50/30 font-medium max-w-xs truncate" title={d.value_b}>
                        {d.value_b !== undefined && d.value_b !== null ? d.value_b : '-'}
                      </td>
                      {/* Row A */}
                      <td className="py-2 px-3 text-right text-slate-500">
                        {d.row_a ?? '-'}
                      </td>
                      {/* Row B */}
                      <td className="py-2 px-3 text-right text-slate-500">
                        {d.row_b ?? '-'}
                      </td>
                      {/* Diff Type */}
                      <td className="py-2 px-3 text-center font-sans">
                        <span className={`px-2 py-0.5 rounded-full text-[10px] font-bold border ${getBadgeStyle(d.difference_type)}`}>
                          {d.difference_type}
                        </span>
                      </td>
                    </tr>
                  );
                })}
                {currentPageDiffs.length === 0 && (
                  <tr>
                    <td colSpan={result.key_columns.length + 6} className="py-8 text-center text-slate-400 font-sans">
                      {result.differences.length === 0
                        ? '恭喜！兩個檔案完全無差異。'
                        : '無符合搜尋條件的差異項目。'}
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          <div className="flex items-center justify-between text-xs text-slate-500 pt-2">
            <div>
              顯示第 {(page - 1) * pageSize + 1} 至{' '}
              {Math.min(page * pageSize, filteredDiffs.length)} 筆（共{' '}
              {filteredDiffs.length.toLocaleString()} 筆）
            </div>

            <div className="flex items-center gap-3">
              <div className="flex items-center gap-1">
                <span>每頁：</span>
                <select
                  value={pageSize}
                  onChange={(e) => {
                    setPageSize(Number(e.target.value));
                    setPage(1);
                  }}
                  className="bg-white border border-slate-200 rounded px-1.5 py-0.5"
                >
                  <option value={50}>50 筆</option>
                  <option value={100}>100 筆</option>
                  <option value={200}>200 筆</option>
                </select>
              </div>

              <div className="flex items-center gap-1">
                <button
                  type="button"
                  disabled={page <= 1}
                  onClick={() => setPage((p: number) => Math.max(1, p - 1))}
                  className="p-1 rounded bg-slate-100 hover:bg-slate-200 disabled:opacity-40"
                >
                  <ChevronLeft className="w-4 h-4" />
                </button>
                <span className="font-semibold text-slate-700 px-2">
                  {page} / {totalPages}
                </span>
                <button
                  type="button"
                  disabled={page >= totalPages}
                  onClick={() => setPage((p: number) => Math.min(totalPages, p + 1))}
                  className="p-1 rounded bg-slate-100 hover:bg-slate-200 disabled:opacity-40"
                >
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab: Column Differences */}
      {activeTab === 'columns' && (
        <div className="overflow-x-auto border border-slate-200 rounded-lg">
          <table className="w-full text-xs text-left">
            <thead className="bg-slate-100 text-slate-700 font-bold border-b border-slate-200">
              <tr>
                <th className="py-2.5 px-4">欄位名稱 (Column Name)</th>
                <th className="py-2.5 px-4">File A 是否存在</th>
                <th className="py-2.5 px-4">File B 是否存在</th>
                <th className="py-2.5 px-4 text-center">狀態 (Status)</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {result.column_differences.map((cd) => (
                <tr key={cd.column} className="hover:bg-slate-50">
                  <td className="py-2.5 px-4 font-semibold text-slate-800">{cd.column}</td>
                  <td className="py-2.5 px-4 font-medium">
                    {cd.file_a ? (
                      <span className="text-emerald-600">✓ 存在</span>
                    ) : (
                      <span className="text-slate-400">✗ 不存在</span>
                    )}
                  </td>
                  <td className="py-2.5 px-4 font-medium">
                    {cd.file_b ? (
                      <span className="text-emerald-600">✓ 存在</span>
                    ) : (
                      <span className="text-slate-400">✗ 不存在</span>
                    )}
                  </td>
                  <td className="py-2.5 px-4 text-center">
                    <span
                      className={`px-2 py-0.5 rounded-full text-[10px] font-bold border ${
                        cd.status === 'SAME'
                          ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                          : cd.status === 'A_ONLY'
                          ? 'bg-amber-50 text-amber-700 border-amber-200'
                          : 'bg-sky-50 text-sky-700 border-sky-200'
                      }`}
                    >
                      {cd.status}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Tab: Duplicate Keys */}
      {activeTab === 'duplicates' && (
        <div className="overflow-x-auto border border-slate-200 rounded-lg">
          <table className="w-full text-xs text-left font-mono">
            <thead className="bg-slate-100 text-slate-700 font-bold border-b border-slate-200 font-sans">
              <tr>
                <th className="py-2.5 px-4">來源檔案</th>
                {result.key_columns.map((k) => (
                  <th key={k} className="py-2.5 px-4">{k}</th>
                ))}
                <th className="py-2.5 px-4 text-right">重複次數</th>
                <th className="py-2.5 px-4">出現在資料列號 (Rows)</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {result.duplicate_key_records.map((dup, idx) => {
                const map = new Map(dup.key_values.map((kv) => [kv.column, kv.value]));
                return (
                  <tr key={idx} className="hover:bg-slate-50">
                    <td className="py-2.5 px-4 font-sans font-semibold text-purple-700">{dup.source}</td>
                    {result.key_columns.map((k) => (
                      <td key={k} className="py-2.5 px-4 text-slate-800">{map.get(k) || '-'}</td>
                    ))}
                    <td className="py-2.5 px-4 text-right font-bold text-rose-600">{dup.count} 次</td>
                    <td className="py-2.5 px-4 text-slate-600">{dup.rows.join(', ')}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};
