import React from 'react';
import {
  CheckCircle2,
  XCircle,
  Clock,
} from 'lucide-react';
import { CompareResult } from '../types';

interface SummaryCardsProps {
  result: CompareResult;
  numericTolerance: string;
}

export const SummaryCards: React.FC<SummaryCardsProps> = ({ result, numericTolerance }) => {
  return (
    <div className="flex flex-col gap-4">
      {/* Banner status */}
      <div
        className={`p-5 rounded-xl border flex items-center justify-between ${
          result.identical
            ? 'bg-emerald-50/90 border-emerald-300 text-emerald-900'
            : 'bg-rose-50/90 border-rose-300 text-rose-900'
        }`}
      >
        <div className="flex items-center gap-3.5">
          {result.identical ? (
            <CheckCircle2 className="w-8 h-8 text-emerald-600 flex-shrink-0" />
          ) : (
            <XCircle className="w-8 h-8 text-rose-600 flex-shrink-0" />
          )}
          <div>
            <h2 className="text-lg font-bold">
              {result.identical
                ? '兩個檔案比對結果：依目前比對規則相同 ✅'
                : '兩個檔案比對結果：發現差異 ❌'}
            </h2>
            <p className="text-xs text-slate-600 mt-0.5">
              {result.key_columns.length > 0
                ? `比對 Key 結構：${result.key_columns.join(' + ')}`
                : '比對模式：依資料列順序 (Row-by-Row)'}
            </p>
            <p className="text-xs text-slate-600 mt-1 break-all">
              本次數值誤差容許值：{numericTolerance}（|A − B| ≤ 容許值）
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 text-xs font-semibold px-3 py-1.5 rounded-lg bg-white/80 border border-slate-200 text-slate-700 shadow-sm">
          <Clock className="w-3.5 h-3.5 text-slate-500" />
          耗時 {result.duration_ms} 毫秒
        </div>
      </div>

      {/* Grid of counters */}
      <div className="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-8 gap-2.5">
        {/* Rows A */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-slate-500">File A 總列數</span>
          <span className="text-lg font-bold text-slate-800 mt-1">
            {result.rows_a.toLocaleString()}
          </span>
        </div>

        {/* Rows B */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-slate-500">File B 總列數</span>
          <span className="text-lg font-bold text-slate-800 mt-1">
            {result.rows_b.toLocaleString()}
          </span>
        </div>

        {/* Same Records */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-emerald-600">相同筆數 (Same)</span>
          <span className="text-lg font-bold text-emerald-700 mt-1">
            {result.same_records.toLocaleString()}
          </span>
        </div>

        {/* Different Records */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-rose-600">相異筆數 (Diff)</span>
          <span className="text-lg font-bold text-rose-700 mt-1">
            {result.different_records.toLocaleString()}
          </span>
        </div>

        {/* Different Cells */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-amber-600">相異儲存格數</span>
          <span className="text-lg font-bold text-amber-700 mt-1">
            {result.different_cells.toLocaleString()}
          </span>
        </div>

        {/* A Only */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-amber-600">A Only (A獨有)</span>
          <span className="text-lg font-bold text-amber-700 mt-1">
            {result.a_only_records.toLocaleString()}
          </span>
        </div>

        {/* B Only */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-sky-600">B Only (B獨有)</span>
          <span className="text-lg font-bold text-sky-700 mt-1">
            {result.b_only_records.toLocaleString()}
          </span>
        </div>

        {/* Duplicate Keys */}
        <div className="bg-white p-3 rounded-lg border border-slate-200 shadow-sm flex flex-col">
          <span className="text-[11px] font-medium text-purple-600">重複 Key 數</span>
          <span className="text-lg font-bold text-purple-700 mt-1">
            {(result.duplicate_keys_a + result.duplicate_keys_b).toLocaleString()}
          </span>
        </div>
      </div>
    </div>
  );
};
