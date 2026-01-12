digraph {
  node [shape=plaintext]
  subgraph {
	_1 [label="1. Record architecture decisions"; URL="0001-record-architecture-decisions.html"];
	_1 [label="ADR 001: Provider Pattern Architecture"; URL="001-provider-pattern-architecture.html"];
	_2 [label="ADR 002: Async-First Architecture"; URL="002-async-first-architecture.html"];
	_1 -> _2 [style="dotted", weight=1];
	_3 [label="ADR 003: C4 Model Documentation"; URL="003-c4-model-documentation.html"];
	_2 -> _3 [style="dotted", weight=1];
	_4 [label="ADR 004: Multi-Provider Strategy"; URL="004-multi-provider-strategy.html"];
	_3 -> _4 [style="dotted", weight=1];
	_5 [label="ADR 005: Documentation Excellence v0.1.0"; URL="005-documentation-excellence-v0.1.0.html"];
	_4 -> _5 [style="dotted", weight=1];
	_6 [label="ADR 006: Code Audit and Architecture Improvements v0.1.0"; URL="006-code-audit-and-improvements-v0.1.0.html"];
	_5 -> _6 [style="dotted", weight=1];
  }
}
