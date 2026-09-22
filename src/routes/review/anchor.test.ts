import { describe, expect, it } from 'vitest';
import { resolveAnchor } from './anchor';

const lines = ['fn main() {', '  let x = foo()?;', '  println!("{x}");', '}'];

describe('resolveAnchor', () => {
  it('leaves the line numbers alone when nothing moved', () => {
    expect(resolveAnchor(lines, { lineStart: 2, anchorText: ['  let x = foo()?;'] })).toEqual({
      kind: 'anchored',
      lineStart: 2,
      lineEnd: undefined,
    });
  });

  it('re-anchors to the new position when lines were inserted above', () => {
    const shifted = ['// added', '// added', ...lines];
    expect(resolveAnchor(shifted, { lineStart: 2, anchorText: ['  let x = foo()?;'] })).toEqual({
      kind: 'anchored',
      lineStart: 4,
      lineEnd: undefined,
    });
  });

  it('keeps a multi-line range the same length when it moves', () => {
    const shifted = ['// added', ...lines];
    const anchorText = ['  let x = foo()?;', '  println!("{x}");'];
    expect(resolveAnchor(shifted, { lineStart: 2, anchorText })).toEqual({
      kind: 'anchored',
      lineStart: 3,
      lineEnd: 4,
    });
  });

  it('orphans a comment whose lines were rewritten', () => {
    const rewritten = [lines[0], '  let x = bar()?;', lines[2], lines[3]];
    expect(resolveAnchor(rewritten, { lineStart: 2, anchorText: ['  let x = foo()?;'] })).toEqual({
      kind: 'orphaned',
    });
  });

  it('orphans rather than guess when the lines now appear twice', () => {
    const twice = [lines[0], '  let x = bar()?;', '  let x = foo()?;', '}', '  let x = foo()?;'];
    expect(resolveAnchor(twice, { lineStart: 2, anchorText: ['  let x = foo()?;'] })).toEqual({
      kind: 'orphaned',
    });
  });

  it('keeps the line numbers when the lines never moved but appear elsewhere too', () => {
    const twice = [...lines, '  let x = foo()?;'];
    expect(resolveAnchor(twice, { lineStart: 2, anchorText: ['  let x = foo()?;'] })).toEqual({
      kind: 'anchored',
      lineStart: 2,
      lineEnd: undefined,
    });
  });

  it('orphans a comment on a file that is gone', () => {
    expect(resolveAnchor(null, { lineStart: 2, anchorText: ['  let x = foo()?;'] })).toEqual({
      kind: 'orphaned',
    });
  });

  it('orphans a comment whose lines only moved past the end of a shortened file', () => {
    expect(resolveAnchor(['fn main() {}'], { lineStart: 2, anchorText: ['  let x = foo()?;'] })).toEqual({
      kind: 'orphaned',
    });
  });
});
