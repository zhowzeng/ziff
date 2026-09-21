# Fetch never prompts, and gives up after a minute

`Fetch` is the only thing Ziff does that leaves the machine, and ADR 0003 already accepted that it borrows the reviewer's own setup to do it: gix shells out to the system `ssh` binary, and HTTPS credentials come from git's credential-helper protocol. Both of those can ask a question — a key passphrase, a password, an unknown host key — and Ziff has nowhere to type the answer. A GUI app with no terminal either fails outright or, worse, sits on a prompt no one will ever see, which the reviewer experiences as a spinner that never stops.

So the helper cascade runs with prompting disabled: a helper that already holds the credential still works (which is how a GitHub HTTPS remote normally behaves), a configured `askpass` program still gets its turn because it can put a window on screen, and anything that would have fallen through to a terminal prompt fails instead. That leaves the system `ssh` binary, which Ziff does not configure and cannot make promises about, so the fetch also runs on its own thread with a 60-second budget. On timeout gix is asked to interrupt and the reviewer is told what happened. The abandoned thread may outlive that answer — a thread blocked on a prompt cannot be killed — but it holds nothing the rest of the app needs.

Two smaller answers follow from the same place. A Repo with no remote is a repo that was never cloned from anywhere, which is an ordinary state and not a failure, so `Fetch` reports there is nothing to fetch rather than showing an error. And which remote to fetch from is git's question to answer, not Ziff's: gix's `find_fetch_remote` uses the checked-out branch's configured remote and falls back to the only one there is, so a Repo with several remotes behaves the way `git fetch` does in the same folder instead of assuming `origin`.

Reaching the network at all means turning on gix's blocking network client and picking an HTTPS transport. Ziff uses `reqwest` with rustls, which keeps the build free of a system `libcurl` or `openssl` the way ADR 0003 preferred, at the cost of a noticeably larger dependency tree.

## Considered Options

- **Refuse credentials entirely** (hand gix a callback that never authenticates): rejected — it would also refuse the credentials a reviewer's helper already has stored, which is exactly how a GitHub HTTPS remote is normally fetched.
- **Ask for the password in Ziff's own UI**: rejected — it makes a review tool the custodian of git credentials, with storage, lifetime and platform-keychain questions attached, to serve the case where the reviewer's own git setup already fails.
- **Force `ssh -o BatchMode=yes` through `core.sshCommand`**: rejected — the override outranks the reviewer's own `core.sshCommand`, breaking a setup Ziff was supposed to borrow rather than replace. The timeout covers the same hang without touching their configuration.
- **No timeout, and let the OS or the remote end it**: rejected — that is the never-ending spinner, and the one outcome worth spending a thread to avoid.
- **A stall timeout rather than a total one** (give up only when no bytes have arrived for a while): rejected for now — it needs progress plumbed out of gix, and a total budget is the smaller thing that answers the question this issue asked. A large first fetch is where the 60 seconds will chafe.
- **`libcurl` or `native-tls` for HTTPS**: rejected — both bring a system C library back into a build ADR 0003 chose gix partly to keep out of.
