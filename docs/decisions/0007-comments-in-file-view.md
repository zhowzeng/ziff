# Comments in File View anchor to worktree line numbers

A [[Comment]] can be left in [[File View]], not only on a diff line. An unchanged file has no diff, so selecting one in the sidebar (via "顯示所有檔案") opens File View — and without commenting there, that checkbox would only ever be a way to browse the project, never a way to say anything about what it shows.

File View has one numbering space — the file's own worktree line numbers — where the diff has two. So a comment's anchor `side` gains a third value, `file`, alongside `new` and `old` (`helpers.ts: CommentAnchor`) — ADR 0010 later dropped `old`, leaving `new` and `file`, but for the same reason the distinction is drawn here. Keeping it a distinct side rather than reusing `new` means a File View comment and a diff comment on the same numbers can never be mistaken for the same anchor, which is what `commentQueue.find` and the commented-line markers key off. The text handed to a CLI agent is unchanged either way — `path:L12`, since the agent reads the worktree too.

## Considered Options

- **Read-only File View** (no commenting, matching the narrower "anchored to a diff line" reading of [[Comment]]): rejected — it leaves "顯示所有檔案" as browsing only, and a reviewer who spots a problem in an unchanged file has nowhere to put the note.
- **Reuse `side: 'new'` for File View comments**: rejected — the two numbering spaces would collide silently on any file that is both changed and browsed, and the queue would treat the two anchors as one.
- **Drop the "顯示所有檔案" checkbox instead**: rejected — the surrounding context an unchanged file carries is often what a review comment needs to point at.
