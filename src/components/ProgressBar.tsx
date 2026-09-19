import React from 'react';
import { Loader2, XCircle } from 'lucide-react';
import { ProgressPayload } from '../types';

interface ProgressBarProps {
  progress: ProgressPayload | null;
  onCancel: () => void;
  cancelling: boolean;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  onCancel,
  cancelling,
}) => {
  const percent = progress?.percent ?? null;
  const processed = progress?.processed_b || progress?.processed_a || 0;
  const total = progress?.total_b || progress?.total_a || null;

  return (
    <div className="bg-white rounded-xl shadow-lg border border-blue-100 p-6 flex flex-col gap-4 animate-in fade-in duration-300">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Loader2 className="w-5 h-5 text-blue-600 animate-spin" />
          <div>
            <h3 className="font-bold text-slate-800 text-sm">
              {progress?.stage || '正在進行資料比對...'}
            </h3>
            <p className="text-xs text-slate-500 mt-0.5">
              {progress?.message || '後端串流處理中，UI 保持順暢，隨時可安全取消'}
            </p>
          </div>
        </div>

        <button
          type="button"
          disabled={cancelling}
          onClick={onCancel}
          className="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg border border-red-200 bg-red-50 text-red-600 hover:bg-red-100 text-xs font-semibold transition disabled:opacity-50"
        >
          <XCircle className="w-3.5 h-3.5" />
          {cancelling ? '正在取消...' : '取消比較'}
        </button>
      </div>

      {/* Progress bar line */}
      <div className="w-full bg-slate-100 rounded-full h-3 overflow-hidden border border-slate-200">
        {percent !== null ? (
          <div
            className="bg-blue-600 h-full rounded-full transition-all duration-300 ease-out"
            style={{ width: `${percent}%` }}
          />
        ) : (
          <div className="bg-blue-500 h-full w-1/3 rounded-full animate-[shimmer_1.5s_infinite] bg-gradient-to-r from-blue-400 via-blue-600 to-blue-400" />
        )}
      </div>

      {/* Statistics */}
      <div className="flex justify-between text-xs text-slate-500 font-medium">
        <span>
          已處理：<strong className="text-slate-700">{processed.toLocaleString()}</strong> 筆資料
          {total ? ` / ${total.toLocaleString()}` : ''}
        </span>
        {percent !== null && (
          <span className="font-bold text-blue-600">{percent}%</span>
        )}
      </div>
    </div>
  );
};
