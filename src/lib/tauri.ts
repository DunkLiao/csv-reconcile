import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import {
  CompareOptions,
  CompareResult,
  FileInfo,
  ParseOptions,
  ProgressPayload,
} from '../types';

export async function inspectFile(
  path: string,
  options?: ParseOptions
): Promise<FileInfo> {
  return await invoke<FileInfo>('inspect_file', { path, options });
}

export async function compareFiles(
  options: CompareOptions
): Promise<CompareResult> {
  return await invoke<CompareResult>('compare_files', { options });
}

export async function cancelCompare(): Promise<void> {
  return await invoke<void>('cancel_compare');
}

export async function exportExcel(
  outputPath: string,
  options: CompareOptions,
  result: CompareResult
): Promise<void> {
  return await invoke<void>('export_excel', {
    outputPath,
    options,
    result,
  });
}

export async function openCsvDialog(): Promise<string | null> {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Delimited Files (*.csv, *.txt)',
          extensions: ['csv', 'txt'],
        },
        {
          name: 'All Files (*.*)',
          extensions: ['*'],
        },
      ],
    });
    if (typeof selected === 'string') {
      return selected;
    }
    return null;
  } catch (err) {
    console.error('Failed to open file dialog:', err);
    return null;
  }
}

export async function saveExcelDialog(): Promise<string | null> {
  try {
    const selected = await save({
      defaultPath: 'CSV_Compare_Result.xlsx',
      filters: [
        {
          name: 'Excel Workbook (*.xlsx)',
          extensions: ['xlsx'],
        },
      ],
    });
    return selected;
  } catch (err) {
    console.error('Failed to save file dialog:', err);
    return null;
  }
}

export async function listenCompareProgress(
  callback: (payload: ProgressPayload) => void
): Promise<UnlistenFn> {
  return await listen<ProgressPayload>('compare-progress', (event) => {
    callback(event.payload);
  });
}
