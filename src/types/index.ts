export type EncodingOption =
  | 'auto'
  | 'utf8'
  | 'utf8_bom'
  | 'big5'
  | 'cp950'
  | 'utf16_le'
  | 'utf16_be';

export type DelimiterOption =
  | { type: 'Auto' }
  | { type: 'Comma' }
  | { type: 'Semicolon' }
  | { type: 'Tab' }
  | { type: 'Pipe' }
  | { type: 'Colon' }
  | { type: 'Custom'; value: string };

export interface ParseOptions {
  encoding: EncodingOption;
  delimiter: DelimiterOption;
}

export interface FileInfo {
  path: string;
  encoding: string;
  delimiter: string;
  delimiter_char: string;
  headers: string[];
  row_count?: number;
}

export type ComparisonMode = 'key_based' | 'row_by_row';

export interface CompareOptions {
  file_a_path: string;
  file_b_path: string;
  file_a_parse_options: ParseOptions;
  file_b_parse_options: ParseOptions;
  comparison_mode: ComparisonMode;
  key_columns: string[];
  excluded_columns: string[];
  trim_whitespace: boolean;
  ignore_case: boolean;
}

export type DifferenceType =
  | 'VALUE_CHANGED'
  | 'A_ONLY'
  | 'B_ONLY'
  | 'COLUMN_A_ONLY'
  | 'COLUMN_B_ONLY'
  | 'DUPLICATE_KEY_A'
  | 'DUPLICATE_KEY_B'
  | 'EMPTY_KEY_A'
  | 'EMPTY_KEY_B'
  | 'INCOMPLETE_KEY_A'
  | 'INCOMPLETE_KEY_B';

export interface KeyValue {
  column: string;
  value: string;
}

export interface Difference {
  key_values: KeyValue[];
  row_a?: number;
  row_b?: number;
  column_name?: string;
  value_a?: string;
  value_b?: string;
  difference_type: DifferenceType;
}

export interface DuplicateKeyRecord {
  source: string;
  key_values: KeyValue[];
  count: number;
  rows: number[];
}

export interface ColumnDifference {
  column: string;
  file_a: boolean;
  file_b: boolean;
  status: 'SAME' | 'A_ONLY' | 'B_ONLY';
}

export interface CompareResult {
  identical: boolean;
  rows_a: number;
  rows_b: number;
  key_columns: string[];
  compared_columns: string[];
  excluded_columns: string[];
  matched_records: number;
  same_records: number;
  different_records: number;
  a_only_records: number;
  b_only_records: number;
  duplicate_keys_a: number;
  duplicate_keys_b: number;
  different_cells: number;
  differences: Difference[];
  column_differences: ColumnDifference[];
  duplicate_key_records: DuplicateKeyRecord[];
  duration_ms: number;
}

export interface ProgressPayload {
  stage: string;
  processed_a: number;
  processed_b: number;
  total_a?: number;
  total_b?: number;
  percent?: number;
  message: string;
}
