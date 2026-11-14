---
name: work
description: Take a GitHub issue of styletto from analysis to a merged pull request - alternatives discussed with the user, implementation, headless screenshots of the stories before and after, review - when asked to work on an issue (/work <issue number>).
argument-hint: <issue number>
---

# Work on an issue

Take the issue the user named (in Claude Code it is `$ARGUMENTS`) to a pull request. The user
decides at two points: the approach, and how it looks. In between, work on your own. Talk to
the user in the language they write in; write code, commits, PRs and issue comments in
English. The repository's rules are in `AGENTS.md`: follow them throughout.

## 1. Understand

- `gh issue view <number> --comments`: the issue and what was already said on it. Its labels
  say what kind of work it is:
  - `look`: how something appears changes;
  - `component`: a new component, or a new variant of one;
  - `api`: the public API changes (other projects depend on it);
  - `tech-debt`: nothing visible changes: every story must draw the same pixels.
- Read the code involved, the README sections that describe it, and its stories.
- If the issue is out of date (already done, or no longer true), say so and propose to close
  it.
- For `look` and `tech-debt` work, take the "before" shots now (see step 4), on `main`.

## 2. Propose, then stop

Give 2-3 real alternatives (not one idea and two straw men). For each one, say what it costs
(code, API changes, new tokens, risk) and what it gives, and say which one you'd pick and why.
When the choice is visual, show it: quick prototypes drawn with `tools/shot.sh`, never ASCII.
When the API changes, show the call site an app would write. Look at how other design systems
solved the same problem when it isn't obvious (Material 3, Apple HIG, Radix, shadcn).

Post the analysis as a comment on the issue (`gh issue comment`), then **end the turn and wait
for the user**. Don't write code before they choose. If the discussion changes the plan, add a
short comment with the decision.

## 3. Implement

- `git switch main && git pull --ff-only`, then a branch `<number>-<short-slug>`.
- Keep to the rules of the system in `AGENTS.md`: no numbers in components, everything from
  the `Ui`; a story for anything new; the README updated with what it describes.
- Keep the change to what the issue asks: anything else you notice goes to the final report,
  not into this branch.

## 4. Check, iterate

Run the checks of `AGENTS.md` and fix until they are clean.

Then look at the result. With `tools/shot.sh`, draw every story the change touches in
the same matrix before and after:

- `mode=dark` and `mode=light`;
- `density=compact`, `default`, `comfortable` and `shape=square`, `soft`, `round`, in at
  least the corners of that grid (compact+square, comfortable+round);
- `material=glass` when the island or anything translucent is involved;
- a second theme (for example `theme=gruvbox`) when colours are involved;
- the story's own controls (`--set`) for the states that matter: disabled, selected, long
  text, empty.

Use the same arguments before and after. Look at the shots yourself first. Then:

- iterate on your own while the result is objectively wrong: clipped or overflowing text,
  misaligned items, lost contrast, corners that don't nest, a regression in another story,
  failing checks;
- taste is the user's: for `look` and `component` work, show the before/after and **wait for
  their ok** before going on;
- for `tech-debt`, prove that nothing visible changed: `magick compare -metric AE` gives 0 for
  every pair.

## 5. Simplify and review

Review the branch's whole diff against `main` twice: once for simplification (reuse of what the
crate already has, code that can go, clearer names), once for correctness (bugs, regressions,
broken API, numbers that should be tokens, a `Ui` choice that isn't followed). In Claude Code
use `/simplify` and then `/code-review`; elsewhere, use your tool's review command if it has
one, or do the two passes yourself.

Act only on what really matters: bugs, regressions, rule breaks, code that will mislead the
next reader. Leave out style preferences and speculative refactors. Re-run the checks and the
shots after fixing; repeat the round only if the fixes were substantial (two rounds at most).

What the review finds in what this change brought in is fixed in this same pull request,
unless it is too large for it: then say so and propose an issue. Problems that were there
before and that the change only brought to light become issues, not part of this branch.

What you leave out that is worth doing later becomes an issue, after the user agrees:
`gh issue create --label tech-debt --label size:S|M|L` with a body that stands on its own
(where, what, why it was left).

## 6. Pull request

- One commit (or a few that each make sense alone): a subject that says what changed, a body
  with one plain-English paragraph per topic. No trailers of any kind.
- `git push -u origin <branch>`, then `gh pr create` with: what changed and why, the
  alternative chosen, how it was checked (commands, stories and settings shot), any API break
  spelled out, what was left out (with the new issues' numbers), and `Closes #<number>`.
  Screenshots can't be attached from the command line: describe them, the user has seen them.
- Wait for CI (`gh pr checks --watch`). If it fails, fix and push again.

## 7. Merge

Merge only with green CI and only when the user says so: `gh pr merge --squash
--delete-branch`, then `git switch main && git pull --ff-only`. Never push to `main` directly.
