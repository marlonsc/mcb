//! Hybrid Rule Engines
//!
//! Provides a unified interface for multiple rule engines:
//! - rust-rule-engine: RETE-UL algorithm with GRL syntax
//! - rusty-rules: JSON DSL with composition (all/any/not)
//! - evalexpr: Simple boolean expression evaluation
//! - validator/garde: Field-level validations
//!
//! ## Engine Selection
//!
//! The router automatically selects the appropriate engine based on rule content:
//! - Rules with "when"/"then" keywords -> RETE engine (GRL syntax)
//! - Rules with "expression" field -> Expression engine (evalexpr)
//! - Rules with "condition"/"action" -> Rusty Rules engine (JSON DSL)

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
