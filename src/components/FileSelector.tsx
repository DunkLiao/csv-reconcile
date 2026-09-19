import React, { useState } from 'react';
import {
  FolderOpen,
  FileSpreadsheet,
  RefreshCw,
  AlertCircle,
  CheckCircle2,
  Settings2,
} from 'lucide-react';
import { DelimiterOption, EncodingOption, FileInfo, ParseOptions } from '../types';
import { openCsvDialog } from '../lib/tauri';

interface FileSelectorProps {
  label: string;
  badgeColor: string;
  path: string;
  setPath: (val: string) => void;
  parseOptions: ParseOptions;
  setParseOptions: (opts: ParseOptions) => void;
  fileInfo: FileInfo | null;
  loading: boolean;
  error: string | null;
  onInspect: () => void;
}

export const FileSelector: React.FC<FileSelectorProps> = ({
  label,
  badgeColor,
  path,
  setPath,
  parseOptions,
  setParseOptions,
  fileInfo,
  loading,
  error,
  onInspect,
}) => {
  const [customChar, setCustomChar] = useState('');

  const handleBrowse = async () => {
    const selected = await openCsvDialog();
    if (selected) {
      setPath(selected);
    }
  };

  const handleEncodingChange = (val: string) => {
    setParseOptions({
      ...parseOptions,
      encoding: val as EncodingOption,
    });
  };

  const handleDelimiterChange = (val: string) => {
    let delim: DelimiterOption;
    switch (val) {
      case 'Comma':
        delim = { type: 'Comma' };
        break;
      case 'Semicolon':
        delim = { type: 'Semicolon' };
        break;
      case 'Tab':
        delim = { type: 'Tab' };
        break;
      case 'Pipe':
        delim = { type: 'Pipe' };
        break;
      case 'Colon':
        delim = { type: 'Colon' };
        break;
      case 'Custom':
        delim = { type: 'Custom', value: customChar || '^' };
        break;
      default:
        delim = { type: 'Auto' };
    }
    setParseOptions({
      ...parseOptions,
      delimiter: delim,
    });
  };

  const currentDelimiterValue = () => {
    if (parseOptions.delimiter.type === 'Custom') return 'Custom';
    return parseOptions.delimiter.type;
  };

  return (
    <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-5 flex flex-col gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <span className={`px-2.5 py-0.5 rounded text-xs font-bold text-white ${badgeColor}`}>
            {label}
          </span>
          <span className="font-semibold text-slate-800 text-sm">來源檔案設定</span>
        </div>
        {fileInfo && !loading && (
          <span className="inline-flex items-center gap-1 text-xs font-medium text-emerald-600 bg-emerald-50 px-2 py-0.5 rounded-full border border-emerald-200">
            <CheckCircle2 className="w-3.5 h-3.5" />
            已成功解析
          </span>
        )}
      </div>

      {/* Path input & Browse */}
      <div>
        <label className="block text-xs font-medium text-slate-600 mb-1.5">檔案路徑 (.csv, .txt)</label>
        <div className="flex gap-2">
          <div className="relative flex-1">
            <input
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="請選擇或貼上來源檔案完整路徑..."
              className="w-full pl-3 pr-8 py-2 text-sm bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white text-slate-700"
            />
            {path && (
              <button
                type="button"
                onClick={() => setPath('')}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 text-xs"
              >
                ✕
              </button>
            )}
          </div>
          <button
            type="button"
            onClick={handleBrowse}
            className="inline-flex items-center gap-1.5 px-3.5 py-2 text-sm font-medium bg-slate-100 hover:bg-slate-200 border border-slate-300 rounded-lg text-slate-700 transition"
          >
            <FolderOpen className="w-4 h-4 text-slate-500" />
            瀏覽
          </button>
        </div>
      </div>

      {/* Encoding & Delimiter settings */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 border-t border-slate-100">
        <div>
          <label className="block text-xs font-medium text-slate-600 mb-1">
            文字編碼 (Encoding)
          </label>
          <select
            value={parseOptions.encoding}
            onChange={(e) => handleEncodingChange(e.target.value)}
            className="w-full py-1.5 px-2.5 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-slate-700 font-mono"
          >
            <option value="auto">
              Auto 自動偵測 {fileInfo ? `(${fileInfo.encoding})` : ''}
            </option>
            <option value="utf8">UTF-8</option>
            <option value="utf8_bom">UTF-8 BOM</option>
            <option value="cp950">CP950 (繁體中文 Windows / Excel)</option>
            <option value="big5">Big5 (繁體中文標準)</option>
            <option value="utf16_le">UTF-16 LE</option>
            <option value="utf16_be">UTF-16 BE</option>
          </select>
        </div>

        <div>
          <label className="block text-xs font-medium text-slate-600 mb-1">
            分隔符號 (Delimiter)
          </label>
          <select
            value={currentDelimiterValue()}
            onChange={(e) => handleDelimiterChange(e.target.value)}
            className="w-full py-1.5 px-2.5 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-slate-700 font-mono"
          >
            <option value="Auto">
              Auto 自動偵測 {fileInfo ? `(${fileInfo.delimiter})` : ''}
            </option>
            <option value="Comma">逗號 Comma (,)</option>
            <option value="Pipe">管線符號 Pipe (|)</option>
            <option value="Tab">定位鍵 Tab (\t)</option>
            <option value="Semicolon">分號 Semicolon (;)</option>
            <option value="Colon">冒號 Colon (:)</option>
            <option value="Custom">自訂字元 (Custom)</option>
          </select>
        </div>
      </div>

      {/* Custom delimiter char input */}
      {parseOptions.delimiter.type === 'Custom' && (
        <div className="flex items-center gap-2 bg-blue-50/50 p-2 rounded-lg border border-blue-100">
          <Settings2 className="w-4 h-4 text-blue-600" />
          <span className="text-xs text-blue-900 font-medium">自訂單一 Unicode 字元：</span>
          <input
            type="text"
            maxLength={1}
            value={customChar}
            onChange={(e) => {
              setCustomChar(e.target.value);
              setParseOptions({
                ...parseOptions,
                delimiter: { type: 'Custom', value: e.target.value || '^' },
              });
            }}
            placeholder="^"
            className="w-12 px-2 py-1 text-center font-mono text-xs border border-blue-300 rounded focus:ring-1 focus:ring-blue-500 focus:outline-none bg-white"
          />
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="flex items-start gap-2 bg-red-50 text-red-700 text-xs p-2.5 rounded-lg border border-red-200">
          <AlertCircle className="w-4 h-4 flex-shrink-0 mt-0.5" />
          <div className="flex-1 font-medium">{error}</div>
        </div>
      )}

      {/* File Info Meta */}
      {fileInfo && !loading && (
        <div className="bg-slate-50 p-2.5 rounded-lg border border-slate-200 flex flex-wrap items-center justify-between text-xs text-slate-600">
          <div className="flex items-center gap-3">
            <span className="inline-flex items-center gap-1 font-medium text-slate-700">
              <FileSpreadsheet className="w-3.5 h-3.5 text-blue-500" />
              欄位數：<strong className="text-blue-700">{fileInfo.headers.length}</strong>
            </span>
            <span>
              總列數：<strong className="text-slate-800">{fileInfo.row_count?.toLocaleString() ?? '未計算'}</strong> 筆
            </span>
          </div>
          <button
            type="button"
            onClick={onInspect}
            className="inline-flex items-center gap-1 text-slate-500 hover:text-blue-600 font-medium"
          >
            <RefreshCw className="w-3 h-3" />
            重新偵測
          </button>
        </div>
      )}

      {loading && (
        <div className="flex items-center justify-center gap-2 py-3 text-xs text-blue-600 bg-blue-50/60 rounded-lg animate-pulse">
          <RefreshCw className="w-3.5 h-3.5 animate-spin" />
          正在分析檔案編碼、分隔符與標題列...
        </div>
      )}
    </div>
  );
};
