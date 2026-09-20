import React, { useState, useMemo } from 'react';
import {
  KeyRound,
  Filter,
  SlidersHorizontal,
  Search,
  CheckSquare,
  Square,
  Play,
  ArrowRightLeft,
  Layers,
  AlertTriangle,
} from 'lucide-react';
import { ComparisonMode, FileInfo } from '../types';

interface CompareConfigProps {
  fileInfoA: FileInfo | null;
  fileInfoB: FileInfo | null;
  mode: ComparisonMode;
  setMode: (m: ComparisonMode) => void;
  keyColumns: string[];
  setKeyColumns: (cols: string[]) => void;
  excludedColumns: string[];
  setExcludedColumns: (cols: string[]) => void;
  trimWhitespace: boolean;
  setTrimWhitespace: (val: boolean) => void;
  ignoreCase: boolean;
  setIgnoreCase: (val: boolean) => void;
  numericTolerance: string;
  setNumericTolerance: (val: string) => void;
  toleranceError: string | null;
  onStartCompare: () => void;
  canCompare: boolean;
}

export const CompareConfig: React.FC<CompareConfigProps> = ({
  fileInfoA,
  fileInfoB,
  mode,
  setMode,
  keyColumns,
  setKeyColumns,
  excludedColumns,
  setExcludedColumns,
  trimWhitespace,
  setTrimWhitespace,
  ignoreCase,
  setIgnoreCase,
  numericTolerance,
  setNumericTolerance,
  toleranceError,
  onStartCompare,
  canCompare,
}) => {
  const [keySearch, setKeySearch] = useState('');
  const [excludeSearch, setExcludeSearch] = useState('');

  // Determine common columns, A Only columns, B Only columns
  const { commonColumns, aOnlyColumns, bOnlyColumns } = useMemo(() => {
    if (!fileInfoA || !fileInfoB) {
      return { commonColumns: [], aOnlyColumns: [], bOnlyColumns: [] };
    }
    const setA = new Set(fileInfoA.headers);
    const setB = new Set(fileInfoB.headers);

    // Keep File A order for common columns
    const common = fileInfoA.headers.filter((h) => setB.has(h));
    const aOnly = fileInfoA.headers.filter((h) => !setB.has(h));
    const bOnly = fileInfoB.headers.filter((h) => !setA.has(h));

    return { commonColumns: common, aOnlyColumns: aOnly, bOnlyColumns: bOnly };
  }, [fileInfoA, fileInfoB]);

  // Key columns ordered by File A header order
  const orderedKeyColumns = useMemo(() => {
    if (!fileInfoA) return keyColumns;
    return fileInfoA.headers.filter((h) => keyColumns.includes(h));
  }, [fileInfoA, keyColumns]);

  const toggleKey = (col: string) => {
    if (keyColumns.includes(col)) {
      setKeyColumns(keyColumns.filter((c) => c !== col));
    } else {
      // Add and also ensure it's removed from excluded columns
      setKeyColumns([...keyColumns, col]);
      setExcludedColumns(excludedColumns.filter((c) => c !== col));
    }
  };

  const toggleExclude = (col: string) => {
    if (keyColumns.includes(col)) return; // Key column cannot be excluded
    if (excludedColumns.includes(col)) {
      setExcludedColumns(excludedColumns.filter((c) => c !== col));
    } else {
      setExcludedColumns([...excludedColumns, col]);
    }
  };

  // Excluded columns batch actions
  const handleSelectAllExcluded = () => {
    const valid = commonColumns.filter((c) => !keyColumns.includes(c));
    setExcludedColumns(valid);
  };

  const handleClearAllExcluded = () => {
    setExcludedColumns([]);
  };

  const handleInvertExcluded = () => {
    const valid = commonColumns.filter((c) => !keyColumns.includes(c));
    const currentSet = new Set(excludedColumns);
    const inverted = valid.filter((c) => !currentSet.has(c));
    setExcludedColumns(inverted);
  };

  const filteredKeyColumns = commonColumns.filter((c) =>
    c.toLowerCase().includes(keySearch.toLowerCase())
  );

  const filteredExcludeColumns = commonColumns.filter((c) =>
    c.toLowerCase().includes(excludeSearch.toLowerCase())
  );

  return (
    <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-6 flex flex-col gap-6">
      {/* Header Info & Column Alignment Summary */}
      <div className="flex flex-col gap-2">
        <h2 className="text-base font-bold text-slate-800 flex items-center gap-2">
          <SlidersHorizontal className="w-5 h-5 text-blue-600" />
          比對模式與欄位配置
        </h2>

        {/* Column comparison badges */}
        {fileInfoA && fileInfoB && (
          <div className="flex flex-wrap gap-2 text-xs pt-1">
            <span className="px-2.5 py-1 bg-emerald-50 text-emerald-700 rounded-md border border-emerald-200 font-medium">
              共同欄位：{commonColumns.length} 個
            </span>
            {aOnlyColumns.length > 0 && (
              <span className="px-2.5 py-1 bg-amber-50 text-amber-700 rounded-md border border-amber-200 font-medium" title={aOnlyColumns.join(', ')}>
                File A 獨有：{aOnlyColumns.length} 個
              </span>
            )}
            {bOnlyColumns.length > 0 && (
              <span className="px-2.5 py-1 bg-sky-50 text-sky-700 rounded-md border border-sky-200 font-medium" title={bOnlyColumns.join(', ')}>
                File B 獨有：{bOnlyColumns.length} 個
              </span>
            )}
          </div>
        )}
      </div>

      {/* Mode selection */}
      <div className="bg-slate-50 p-4 rounded-xl border border-slate-200">
        <label className="block text-xs font-bold text-slate-700 mb-2 uppercase tracking-wide">
          比對模式 (Comparison Mode)
        </label>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <label
            className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition ${
              mode === 'key_based'
                ? 'bg-blue-50/80 border-blue-400 text-blue-900 shadow-sm'
                : 'bg-white border-slate-200 text-slate-700 hover:bg-slate-100'
            }`}
          >
            <input
              type="radio"
              name="mode"
              checked={mode === 'key_based'}
              onChange={() => setMode('key_based')}
              className="mt-1 text-blue-600 focus:ring-blue-500"
            />
            <div>
              <div className="font-semibold text-sm flex items-center gap-1.5">
                <KeyRound className="w-4 h-4 text-blue-600" />
                使用 Key 比對 (推薦)
              </div>
              <div className="text-xs text-slate-500 mt-0.5">
                依指定的一或多個欄位作為唯一鍵，忽略資料列先後順序，適合報表核對與資料異動比對。
              </div>
            </div>
          </label>

          <label
            className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition ${
              mode === 'row_by_row'
                ? 'bg-blue-50/80 border-blue-400 text-blue-900 shadow-sm'
                : 'bg-white border-slate-200 text-slate-700 hover:bg-slate-100'
            }`}
          >
            <input
              type="radio"
              name="mode"
              checked={mode === 'row_by_row'}
              onChange={() => setMode('row_by_row')}
              className="mt-1 text-blue-600 focus:ring-blue-500"
            />
            <div>
              <div className="font-semibold text-sm flex items-center gap-1.5">
                <ArrowRightLeft className="w-4 h-4 text-slate-600" />
                依資料列順序比對 (Row-by-Row)
              </div>
              <div className="text-xs text-slate-500 mt-0.5">
                第 1 列比對第 1 列、第 2 列比對第 2 列，欄位仍依 Header 名稱自動對齊。
              </div>
            </div>
          </label>
        </div>
      </div>

      {/* Key selection (only for key-based) */}
      {mode === 'key_based' && (
        <div className="bg-slate-50 p-4 rounded-xl border border-slate-200 flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <KeyRound className="w-4 h-4 text-blue-600" />
              <span className="text-sm font-bold text-slate-800">
                Key 關鍵欄位 (單一或複合 Key)
              </span>
              <span className="text-xs text-slate-500">(僅能從兩檔案共同欄位中選取)</span>
            </div>
            {keyColumns.length === 0 && (
              <span className="text-xs font-medium text-amber-600 flex items-center gap-1">
                <AlertTriangle className="w-3.5 h-3.5" />
                請至少勾選一個 Key 欄位
              </span>
            )}
          </div>

          {/* Search Key */}
          <div className="relative">
            <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
            <input
              type="text"
              value={keySearch}
              onChange={(e) => setKeySearch(e.target.value)}
              placeholder="搜尋欄位名稱..."
              className="w-full pl-9 pr-3 py-1.5 text-xs bg-white border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Key columns list */}
          <div className="max-h-48 overflow-y-auto bg-white border border-slate-200 rounded-lg p-3 grid grid-cols-2 sm:grid-cols-3 gap-2">
            {filteredKeyColumns.map((col) => {
              const isChecked = keyColumns.includes(col);
              return (
                <label
                  key={col}
                  className={`flex items-center gap-2 p-1.5 rounded cursor-pointer text-xs transition select-none ${
                    isChecked
                      ? 'bg-blue-50 text-blue-900 font-semibold border border-blue-200'
                      : 'hover:bg-slate-50 text-slate-700'
                  }`}
                >
                  <input
                    type="checkbox"
                    checked={isChecked}
                    onChange={() => toggleKey(col)}
                    className="rounded text-blue-600 focus:ring-blue-500"
                  />
                  <span className="truncate" title={col}>
                    {col}
                  </span>
                </label>
              );
            })}
            {filteredKeyColumns.length === 0 && (
              <div className="col-span-full py-4 text-center text-xs text-slate-400">
                無相符的共同欄位
              </div>
            )}
          </div>

          {/* Composite Key preview */}
          {orderedKeyColumns.length > 0 && (
            <div className="bg-blue-50/70 p-2.5 rounded-lg border border-blue-200 text-xs">
              <span className="font-semibold text-blue-900">Composite Key 結構：</span>
              <span className="font-mono text-blue-700 ml-1">
                {orderedKeyColumns.join(' + ')}
              </span>
            </div>
          )}
        </div>
      )}

      {/* Excluded columns selection */}
      <div className="bg-slate-50 p-4 rounded-xl border border-slate-200 flex flex-col gap-3">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <div className="flex items-center gap-2">
            <Filter className="w-4 h-4 text-slate-600" />
            <span className="text-sm font-bold text-slate-800">排除比較欄位 (Excluded Columns)</span>
            <span className="text-xs text-slate-500">(此區勾選之欄位不影響一致性判定)</span>
          </div>
          {/* Batch operations */}
          <div className="flex items-center gap-1.5 text-xs">
            <button
              type="button"
              onClick={handleSelectAllExcluded}
              className="px-2 py-1 bg-white hover:bg-slate-100 border border-slate-200 rounded text-slate-700 flex items-center gap-1"
            >
              <CheckSquare className="w-3.5 h-3.5 text-slate-500" />
              全選
            </button>
            <button
              type="button"
              onClick={handleClearAllExcluded}
              className="px-2 py-1 bg-white hover:bg-slate-100 border border-slate-200 rounded text-slate-700 flex items-center gap-1"
            >
              <Square className="w-3.5 h-3.5 text-slate-500" />
              全部取消
            </button>
            <button
              type="button"
              onClick={handleInvertExcluded}
              className="px-2 py-1 bg-white hover:bg-slate-100 border border-slate-200 rounded text-slate-700 flex items-center gap-1"
            >
              <Layers className="w-3.5 h-3.5 text-slate-500" />
              反向選取
            </button>
          </div>
        </div>

        {/* Search Exclude */}
        <div className="relative">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            type="text"
            value={excludeSearch}
            onChange={(e) => setExcludeSearch(e.target.value)}
            placeholder="搜尋要排除比較的欄位..."
            className="w-full pl-9 pr-3 py-1.5 text-xs bg-white border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Excluded columns list */}
        <div className="max-h-48 overflow-y-auto bg-white border border-slate-200 rounded-lg p-3 grid grid-cols-2 sm:grid-cols-3 gap-2">
          {filteredExcludeColumns.map((col) => {
            const isKey = keyColumns.includes(col);
            const isChecked = excludedColumns.includes(col);
            return (
              <label
                key={col}
                className={`flex items-center gap-2 p-1.5 rounded text-xs transition select-none ${
                  isKey
                    ? 'opacity-40 cursor-not-allowed bg-slate-100 text-slate-400'
                    : isChecked
                    ? 'bg-amber-50 text-amber-900 font-semibold border border-amber-200 cursor-pointer'
                    : 'hover:bg-slate-50 text-slate-700 cursor-pointer'
                }`}
              >
                <input
                  type="checkbox"
                  disabled={isKey}
                  checked={isChecked && !isKey}
                  onChange={() => toggleExclude(col)}
                  className="rounded text-amber-600 focus:ring-amber-500"
                />
                <span className="truncate" title={isKey ? `${col} (已選為 Key，無法排除)` : col}>
                  {col}
                  {isKey && <span className="ml-1 text-[10px] text-slate-400">(Key)</span>}
                </span>
              </label>
            );
          })}
          {filteredExcludeColumns.length === 0 && (
            <div className="col-span-full py-4 text-center text-xs text-slate-400">
              無相符的共同欄位
            </div>
          )}
        </div>
      </div>

      {/* Value Comparison Rules */}
      <div className="bg-slate-50 p-4 rounded-xl border border-slate-200">
        <label className="block text-xs font-bold text-slate-700 mb-2">比對微調設定 (可選)</label>
        <div className="mb-4">
          <label htmlFor="numeric-tolerance" className="block text-sm font-semibold text-slate-700 mb-2">
            數值誤差容許值
          </label>
          <input
            id="numeric-tolerance"
            type="text"
            inputMode="decimal"
            value={numericTolerance}
            onChange={(e) => setNumericTolerance(e.target.value)}
            aria-invalid={Boolean(toleranceError)}
            aria-describedby="numeric-tolerance-help numeric-tolerance-error"
            className={`w-full sm:w-64 px-3 py-2 text-sm bg-white border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${toleranceError ? 'border-rose-400' : 'border-slate-300'}`}
          />
          <p id="numeric-tolerance-help" className="text-xs text-slate-500 mt-2 leading-relaxed">
            所有參與比對的數值均套用 |A − B| ≤ 容許值（含等於）。預設為 0，001、1.0 與 1 仍視為相同。
            支援小數、科學記號及標準千分位；百分比與貨幣符號依文字比對。Key 配對不受影響。
          </p>
          <p id="numeric-tolerance-error" role={toleranceError ? 'alert' : undefined} className="text-xs text-rose-600 mt-1">
            {toleranceError}
          </p>
        </div>
        <div className="flex flex-wrap gap-6 text-xs text-slate-700">
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={trimWhitespace}
              onChange={(e) => setTrimWhitespace(e.target.checked)}
              className="rounded text-blue-600 focus:ring-blue-500"
            />
            <span>忽略前後空白 (Trim Whitespace)</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={ignoreCase}
              onChange={(e) => setIgnoreCase(e.target.checked)}
              className="rounded text-blue-600 focus:ring-blue-500"
            />
            <span>忽略英文字母大小寫 (Ignore Case)</span>
          </label>
        </div>
        <div className="text-[11px] text-slate-400 mt-2">
          * 雙方皆為有效數值時採數值比對，其餘依文字比對。設定僅影響判定，不會修改原始資料內容、前導零與 Excel 匯出值。
        </div>
      </div>

      {/* Start Button */}
      <div className="pt-2 flex justify-end">
        <button
          type="button"
          disabled={!canCompare}
          onClick={onStartCompare}
          className={`inline-flex items-center gap-2 px-8 py-3 rounded-xl font-bold text-sm shadow-md transition ${
            canCompare
              ? 'bg-blue-600 hover:bg-blue-700 text-white cursor-pointer active:scale-98'
              : 'bg-slate-300 text-slate-500 cursor-not-allowed shadow-none'
          }`}
        >
          <Play className="w-4 h-4 fill-current" />
          開始比對
        </button>
      </div>
    </div>
  );
};
