import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';
import ts from 'typescript';

// Run the actual frontend validator without adding a separate test framework.
const source = await readFile(new URL('../src/lib/numericTolerance.ts', import.meta.url), 'utf8');
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
});
const { numericToleranceError } = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString('base64')}`);

test('accepts nonnegative exact decimal tolerances without Number rounding', () => {
  for (const value of ['0', '-0', '+.5', '1.', '0.1', '1,234.50', '1e-30', '9007199254740993', '1e4096', '1e-4096']) {
    assert.equal(numericToleranceError(value), null, value);
  }
});

test('rejects invalid and negative input before comparison starts', () => {
  for (const value of ['', ' ', ' 1', '1\n', '-0.1', '-1e-999', 'NaN', 'Infinity', '12,34', '10%', '$1', '(1)', '0x10', '1_000', '1e', '1e4097', '1e-4097', '1e999999999', '1'.repeat(4097)]) {
    assert.notEqual(numericToleranceError(value), null, JSON.stringify(value));
  }
});
