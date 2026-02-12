//! # Validation Rule Engines
//!
//! Five engines, each serving a distinct purpose:
//!
//! | Engine | Purpose | External crate |
//! |--------|---------|----------------|
//! | `ReteEngine` | RETE-UL algorithm with GRL when/then syntax | `rust-rule-engine` |
//! | `ExpressionEngine` | Simple boolean expressions (file_count > 500) | `evalexpr` |
//! | `RustyRulesEngineWrapper` | JSON DSL with all/any/not composition | `rusty-rules` |
//! | `ValidatorEngine` | Rule _definition_ validation (field checks) | `validator` |
//! | `RuleEngineRouter` | Auto-detects engine from rule content, dispatches | — |
//!
//! `HybridRuleEngine` is the top-level orchestrator. It owns the router (which
//! owns the three execution engines) plus `ValidatorEngine` and a compiled-rule
//! cache. All rule execution flows through the router — no duplicate engine
//! instances.
//!
//! ## Routing logic (in `RuleEngineRouter`)
//!
//! 1. Explicit `"engine"` field → use specified engine
//! 2. Contains "when"/"then" → RETE
//! 3. Has `"expression"` field → Expression
//! 4. Has `"condition"`/`"action"` → RustyRules
//! 5. Default → RustyRules

pub mod expression_engine;
pub mod hybrid_engine;
pub mod rete_engine;
pub mod router;

pub mod rusty_rules_engine;
pub mod validator_engine;

pub use expression_engine::ExpressionEngine;
pub use hybrid_engine::{
    HybridRuleEngine, RuleContext, RuleEngine, RuleEngineType, RuleResult, RuleViolation,
};
pub use rete_engine::ReteEngine;
pub use router::{RoutedEngine, RuleEngineRouter};

pub use rusty_rules_engine::RustyRulesEngineWrapper;
pub use validator_engine::ValidatorEngine;
