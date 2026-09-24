import { describe, expect, it } from 'vitest';
import { paintLine } from './highlight';
import { languageOf, tokenize } from './tokenize';

describe('languageOf', () => {
  it('goes by the extension', () => {
    expect(languageOf('src/routes/review/state.svelte.ts')).toBe('ts');
    expect(languageOf('src-tauri/src/lib.rs')).toBe('rs');
  });

  it('goes by the whole name when there is no extension', () => {
    expect(languageOf('docker/Dockerfile')).toBe('dockerfile');
  });

  it('has nothing for a file it has no grammar for', () => {
    expect(languageOf('notes.unknownext')).toBeNull();
    // Not a key of the grammar table, even though every object has one by that name.
    expect(languageOf('constructor')).toBeNull();
  });
});

describe('tokenize', () => {
  it('colours a line from what came before it, not just the line itself', async () => {
    const lines = await tokenize('a.ts', 'const a = 1; /* start\nstill a comment\nend */ let b;');
    expect(lines).toHaveLength(3);
    expect(lines![1]).toEqual([{ text: 'still a comment', color: 'var(--syntax-token-comment)' }]);
  });

  it('leaves a file with no grammar as plain text', async () => {
    expect(await tokenize('notes.unknownext', 'hello')).toBeNull();
  });
});

describe('paintLine', () => {
  const tokens = [
    { text: 'let', color: 'k' },
    { text: ' x = ', color: 'p' },
    { text: '42', color: 'c' },
  ];

  it('cuts wherever a colour or a change mark begins', () => {
    const segments = [
      { text: 'let x', changed: false },
      { text: ' = 42', changed: true },
    ];
    expect(paintLine('let x = 42', tokens, segments)).toEqual([
      { text: 'let', changed: false, color: 'k' },
      { text: ' x', changed: false, color: 'p' },
      { text: ' = ', changed: true, color: 'p' },
      { text: '42', changed: true, color: 'c' },
    ]);
  });

  it('draws the line plain when the tokens spell something else', () => {
    expect(paintLine('let y = 42', tokens)).toEqual([{ text: 'let y = 42', changed: false, color: undefined }]);
  });

  it('draws nothing for an empty line', () => {
    expect(paintLine('', [])).toEqual([]);
  });
});
