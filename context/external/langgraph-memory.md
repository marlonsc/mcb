# LangGraph Memory Notes

Last updated: 2026-02-11
Source: LangChain/LangGraph docs (`add-memory`, `persistence`, `long-term-memory`)

Key implementation patterns:

- Separate short-term (thread state + checkpointer) from long-term (store).
- Use namespace tuples for durable memory partitioning (for example `(user_id, "memories")`).
- Store memory as JSON-like records keyed by unique IDs.
- Retrieve memory via semantic `store.search(namespace, query, limit)`.
- Inject only top relevant memories into prompts to control context size.

Transferable guidance for this repository:

- Keep long-term memory records concise and typed.
- Use explicit tenant/session scoping in keys/tags.
- Avoid bulk prompt injection; select small high-signal subsets.
