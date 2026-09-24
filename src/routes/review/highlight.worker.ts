// Runs Shiki off the main thread: a few thousand lines take seconds to tokenize, and
// on the main thread that is seconds of a window that can't scroll or type.

import { tokenize } from './tokenize';

export type HighlightRequest = { id: number; path: string; text: string };
export type HighlightReply = { id: number; tokens: Awaited<ReturnType<typeof tokenize>> } | { id: number; error: string };

self.onmessage = async (e: MessageEvent<HighlightRequest>) => {
  const { id, path, text } = e.data;
  let reply: HighlightReply;
  try {
    reply = { id, tokens: await tokenize(path, text) };
  } catch (err) {
    reply = { id, error: String(err) };
  }
  self.postMessage(reply);
};
