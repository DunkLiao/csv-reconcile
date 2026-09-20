import React, { useState, useEffect, useCallback } from 'react';
import { FileSpreadsheet, ShieldAlert } from 'lucide-react';
import {
  CompareOptions,
  CompareResult,
  ComparisonMode,
  FileInfo,
  ParseOptions,
  ProgressPayload,
} from './types';
import {
  cancelCompare,
  compareFiles,
  inspectFile,
  listenCompareProgress,
} from './lib/tauri';
import { FileSelector } from './components/FileSelector';
import { CompareConfig } from './components/CompareConfig';
import { ProgressBar } from './components/ProgressBar';
import { SummaryCards } from './components/SummaryCards';
import { DifferenceTable } from './components/DifferenceTable';
import { numericToleranceError } from './lib/numericTolerance';

export const App: React.FC = () => {
  // File A State
  const [pathA, setPathA] = useState('');
  const [parseOptionsA, setParseOptionsA] = useState<ParseOptions>({
    encoding: 'auto',
    delimiter: { type: 'Auto' },
  });
  const [fileInfoA, setFileInfoA] = useState<FileInfo | null>(null);
  const [loadingA, setLoadingA] = useState(false);
  const [errorA, setErrorA] = useState<string | null>(null);

  // File B State
  const [pathB, setPathB] = useState('');
  const [parseOptionsB, setParseOptionsB] = useState<ParseOptions>({
    encoding: 'auto',
    delimiter: { type: 'Auto' },
  });
  const [fileInfoB, setFileInfoB] = useState<FileInfo | null>(null);
  const [loadingB, setLoadingB] = useState(false);
  const [errorB, setErrorB] = useState<string | null>(null);

  // Comparison Options State
  const [mode, setMode] = useState<ComparisonMode>('key_based');
  const [keyColumns, setKeyColumns] = useState<string[]>([]);
  const [excludedColumns, setExcludedColumns] = useState<string[]>([]);
  const [trimWhitespace, setTrimWhitespace] = useState(false);
  const [ignoreCase, setIgnoreCase] = useState(false);
  const [numericTolerance, setNumericTolerance] = useState('0');
  const toleranceError = numericToleranceError(numericTolerance);

  // Execution & Results State
  const [isComparing, setIsComparing] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [progress, setProgress] = useState<ProgressPayload | null>(null);
  const [compareResult, setCompareResult] = useState<CompareResult | null>(null);
  const [resultOptions, setResultOptions] = useState<CompareOptions | null>(null);
  const [generalError, setGeneralError] = useState<string | null>(null);

  // Inspect File A
  const handleInspectA = useCallback(async () => {
    if (!pathA.trim()) return;
    setLoadingA(true);
    setErrorA(null);
    try {
      const info = await inspectFile(pathA, parseOptionsA);
      setFileInfoA(info);
    } catch (err: any) {
      setErrorA(err?.message || '無法解析 File A，請確認路徑或編碼格式。');
      setFileInfoA(null);
    } finally {
      setLoadingA(false);
    }
  }, [pathA, parseOptionsA]);

  // Inspect File B
  const handleInspectB = useCallback(async () => {
    if (!pathB.trim()) return;
    setLoadingB(true);
    setErrorB(null);
    try {
      const info = await inspectFile(pathB, parseOptionsB);
      setFileInfoB(info);
    } catch (err: any) {
      setErrorB(err?.message || '無法解析 File B，請確認路徑或編碼格式。');
      setFileInfoB(null);
    } finally {
      setLoadingB(false);
    }
  }, [pathB, parseOptionsB]);

  // Auto-trigger inspection when path or options change
  useEffect(() => {
    if (pathA.trim()) {
      handleInspectA();
    } else {
      setFileInfoA(null);
      setErrorA(null);
    }
  }, [pathA, parseOptionsA, handleInspectA]);

  useEffect(() => {
    if (pathB.trim()) {
      handleInspectB();
    } else {
      setFileInfoB(null);
      setErrorB(null);
    }
  }, [pathB, parseOptionsB, handleInspectB]);

  // Set default keys when both files are inspected
  useEffect(() => {
    if (fileInfoA && fileInfoB) {
      const setB = new Set(fileInfoB.headers);
      const common = fileInfoA.headers.filter((h) => setB.has(h));
      // If no keys selected yet, default to first common column or common ID-like column
      if (keyColumns.length === 0 && common.length > 0) {
        const idCol = common.find((c) =>
          /id|編號|序號|代號|code|no/i.test(c)
        );
        setKeyColumns([idCol || common[0]]);
      }
    }
  }, [fileInfoA, fileInfoB]);

  // Listen to background progress
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenCompareProgress((p) => {
      setProgress(p);
    }).then((un) => {
      unlisten = un;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const canCompare = Boolean(
    fileInfoA &&
      fileInfoB &&
      !loadingA &&
      !loadingB &&
      !toleranceError &&
      (mode === 'row_by_row' || keyColumns.length > 0)
  );

  // Start comparison
  const handleStartCompare = async () => {
    if (!canCompare) return;

    setIsComparing(true);
    setCancelling(false);
    setProgress(null);
    setGeneralError(null);
    setCompareResult(null);
    setResultOptions(null);

    const options: CompareOptions = {
      file_a_path: pathA,
      file_b_path: pathB,
      file_a_parse_options: parseOptionsA,
      file_b_parse_options: parseOptionsB,
      comparison_mode: mode,
      key_columns: keyColumns,
      excluded_columns: excludedColumns,
      trim_whitespace: trimWhitespace,
      ignore_case: ignoreCase,
      numeric_tolerance: numericTolerance,
    };

    try {
      const result = await compareFiles(options);
      setResultOptions(options);
      setCompareResult(result);
    } catch (err: any) {
      setGeneralError(err?.message || '比對失敗，請確認檔案設定與編碼是否正確。');
    } finally {
      setIsComparing(false);
    }
  };

  // Cancel comparison
  const handleCancel = async () => {
    try {
      setCancelling(true);
      await cancelCompare();
    } catch (err) {
      console.error('Failed to cancel compare:', err);
    }
  };

  return (
    <div className="min-h-screen bg-slate-100 flex flex-col selection:bg-blue-500 selection:text-white">
      {/* Top Navigation Bar */}
      <header className="bg-white border-b border-slate-200 sticky top-0 z-30 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-600 to-indigo-700 flex items-center justify-center text-white shadow-md">
              <FileSpreadsheet className="w-6 h-6" />
            </div>
            <div>
              <h1 className="text-base font-extrabold text-slate-900 tracking-tight flex items-center gap-2">
                CSV Compare
                <span className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-blue-100 text-blue-700 border border-blue-200">
                  v1.0 Portable
                </span>
              </h1>
              <p className="text-xs text-slate-500">
                100% 本機離線核對 · 支援繁中 CP950 · 複合 Key · Quoted CSV
              </p>
            </div>
          </div>

        </div>
      </header>

      {/* Main Container */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 flex-1 w-full flex flex-col gap-6">
        {/* General Error Banner */}
        {generalError && (
          <div className="p-4 bg-rose-50 text-rose-800 border border-rose-200 rounded-xl flex items-start gap-3 shadow-sm">
            <ShieldAlert className="w-5 h-5 flex-shrink-0 text-rose-600 mt-0.5" />
            <div>
              <h4 className="font-bold text-sm">比對過程發生錯誤</h4>
              <p className="text-xs mt-0.5 font-medium">{generalError}</p>
            </div>
          </div>
        )}

        {/* File Selectors (File A & File B) */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <FileSelector
            label="File A"
            badgeColor="bg-blue-600"
            path={pathA}
            setPath={setPathA}
            parseOptions={parseOptionsA}
            setParseOptions={setParseOptionsA}
            fileInfo={fileInfoA}
            loading={loadingA}
            error={errorA}
            onInspect={handleInspectA}
          />

          <FileSelector
            label="File B"
            badgeColor="bg-indigo-600"
            path={pathB}
            setPath={setPathB}
            parseOptions={parseOptionsB}
            setParseOptions={setParseOptionsB}
            fileInfo={fileInfoB}
            loading={loadingB}
            error={errorB}
            onInspect={handleInspectB}
          />
        </div>

        {/* Compare in progress */}
        {isComparing && (
          <ProgressBar
            progress={progress}
            onCancel={handleCancel}
            cancelling={cancelling}
          />
        )}

        {/* Results View */}
        {compareResult && resultOptions && !isComparing && (
          <div className="flex flex-col gap-6 animate-in fade-in duration-300">
            <SummaryCards result={compareResult} numericTolerance={resultOptions.numeric_tolerance} />
            <DifferenceTable
              result={compareResult}
              options={resultOptions}
              onReset={() => setCompareResult(null)}
            />
          </div>
        )}

        {/* Configuration Panel (Shown when not showing result) */}
        {!compareResult && !isComparing && (
          <CompareConfig
            fileInfoA={fileInfoA}
            fileInfoB={fileInfoB}
            mode={mode}
            setMode={setMode}
            keyColumns={keyColumns}
            setKeyColumns={setKeyColumns}
            excludedColumns={excludedColumns}
            setExcludedColumns={setExcludedColumns}
            trimWhitespace={trimWhitespace}
            setTrimWhitespace={setTrimWhitespace}
            ignoreCase={ignoreCase}
            setIgnoreCase={setIgnoreCase}
            numericTolerance={numericTolerance}
            setNumericTolerance={setNumericTolerance}
            toleranceError={toleranceError}
            onStartCompare={handleStartCompare}
            canCompare={canCompare}
          />
        )}
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-200 bg-white py-4 text-center text-xs text-slate-500">
        CSV Compare Desktop Tool — 繁體中文報表與資料核對桌面工具 · 100% Local Only · No Cloud
      </footer>
    </div>
  );
};
