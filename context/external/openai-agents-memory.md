# OpenAI Agents Memory Notes

Last updated: 2026-02-11
Source: OpenAI Agents SDK docs and guidance on context engineering

Observed patterns:

- Keep session memory bounded and trimmed to preserve quality.
- Persist durable user/project facts separately from turn-by-turn messages.
- Distill reusable notes from sessions, then re-inject selectively.
- Prefer structured state over raw transcript replay.

Transferable guidance for this repository:

- Split recall into profile/context/pattern searches before execution.
- Store post-task learnings as compact observations with tags.
- Sync stable learnings into markdown context files for human+agent reuse.
