"""Fix strategies for code quality issues."""

import abc


class FixStrategy(abc.ABC):
    """Base for smell-fix strategies."""

    @property
    @abc.abstractmethod
    def rule(self) -> str:
        """Short rule name."""

    @property
    @abc.abstractmethod
    def title(self) -> str:
        """Human-readable title."""

    @property
    @abc.abstractmethod
    def instructions(self) -> str:
        """English fix instructions."""


class IdenticalCodeStrategy(FixStrategy):
    """Strategy for fixing identical code blocks."""

    rule = "identical-code"
    title = "Eliminate identical code blocks"
    instructions = "\\n".join(
        [
            "Refactor duplicated logic into shared abstractions:",
            "- **Domain Logic**: Move shared business rules to `mcb-domain` entities or services.",
            "- **Infrastructure**: Extract common technical implementations to "
            "`mcb-infrastructure::utils`.",
            "- **Tests**: Use `mcb_domain::test_services_config` or shared test fixtures.",
        ]
    )


class SimilarCodeStrategy(FixStrategy):
    """Strategy for fixing similar code blocks."""

    rule = "similar-code"
    title = "Refactor similar code blocks"
    instructions = "\\n".join(
        [
            "Unify similar patterns using Rust's powerful type system:",
            "- **Traits**: Define a trait in `mcb-domain::ports` and implement variations in "
            "`mcb-providers`.",
            "- **Generics**: Use generic parameters for slight variations in types.",
            "- **Macros**: Use `macro_rules!` (sparingly) for structural repetition that generics "
            "can't handle.",
        ]
    )


class FunctionComplexityStrategy(FixStrategy):
    """Strategy for reducing function complexity."""

    rule = "function-complexity"
    title = "Reduce function complexity"
    instructions = "\\n".join(
        [
            "Simplify complex functions by extracting logic:",
            "- **Abstraction**: Move distinct steps into private helper methods or `impl` "
            "blocks.",
            "- **Guard Clauses**: Use `if ... { return ... }` to reduce nesting depth.",
            "- **Pattern Matching**: Use `match` expressions instead of complex `if/else` chains.",
            "- **Error Handling**: Use the `?` operator for clean error propagation.",
        ]
    )


class MethodComplexityStrategy(FixStrategy):
    """Strategy for reducing method complexity."""

    rule = "method-complexity"
    title = "Reduce method complexity"
    instructions = FunctionComplexityStrategy.instructions


class CognitiveComplexityStrategy(FixStrategy):
    """Strategy for reducing cognitive complexity."""

    rule = "cognitive-complexity"
    title = "Lower cognitive complexity"
    instructions = "\\n".join(
        [
            "Make the code easier to reason about:",
            "- **Encapsulation**: Hide complex details behind descriptive function names.",
            "- **Boolean Logic**: Extract complex conditions into `is_valid()` styling methods.",
            "- **Control Flow**: Prefer iterators (`map`, `filter`, `fold`) over manual loops with state.",
        ]
    )


class NestedControlFlowStrategy(FixStrategy):
    """Strategy for flattening nested control flow."""

    rule = "nested-control-flow"
    title = "Flatten deeply nested control flow"
    instructions = "\\n".join(
        [
            "Reduce nesting depth (target ≤ 4 levels):",
            "- **Guard Clauses**: Check preconditions early and return.",
            "- **Iterators**: Use functional combinators to transform collections "
            "flatly.",
            "- **Lets**: Use `let ... = match ...` to assign results instead of nesting logic.",
        ]
    )


class DeepNestingStrategy(FixStrategy):
    """Strategy for flattening deep nesting."""

    rule = "deep-nesting"
    title = "Flatten deep nesting"
    instructions = NestedControlFlowStrategy.instructions


class FileComplexityStrategy(FixStrategy):
    """Strategy for splitting complex files."""

    rule = "file-complexity"
    title = "Split complex file into modules"
    instructions = "\\n".join(
        [
            "Break down large files into focused modules:",
            "- **Modularity**: Create a directory with `mod.rs` and split concerns into separate "
            "files.",
            "- **Clean Architecture**: Ensure the file strictly belongs to one layer (Domain, "
            "Infra, App).",
            "- **Helpers**: Move utility functions to `utils.rs` or specialized submodules.",
        ]
    )


class LongMethodStrategy(FixStrategy):
    """Strategy for shortening long methods."""

    rule = "long-method"
    title = "Shorten long method"
    instructions = "\\n".join(
        [
            "Break methods into single-responsibility steps:",
            "- **Steps**: Identify logical sections (setup, process, output) and extract them.",
            "- **Size**: Aim for methods that fit on a single screen (≤ 25 lines).",
            "- **Context**: If passing many variables, consider a context struct.",
        ]
    )


class LargeClassStrategy(FixStrategy):
    """Strategy for decomposing large classes."""

    rule = "large-class"
    title = "Decompose large struct/class"
    instructions = "\\n".join(
        [
            "Redistribute responsibilities from this large struct:",
            "- **Composition**: Extract groups of fields into smaller Value Objects (in "
            "`mcb-domain::value_objects`).",
            "- **Behavior**: Move complex logic to Domain Services if it involves multiple "
            "entities.",
            "- **Traits**: Implement standard traits (`From`, `TryFrom`, `Display`) to offload "
            "conversion logic.",
        ]
    )


class GodClassStrategy(FixStrategy):
    """Strategy for decomposing god classes."""

    rule = "god-class"
    title = "Decompose God Class"
    instructions = "\\n".join(
        [
            "This struct violates Single Responsibility Principle:",
            "- **Domain Services**: Split orchestration logic into specific Application Services.",
            "- **Rich Entities**: Move business rules to the Entities that hold the data.",
            "- **Providers**: Delegate external interaction to `mcb-providers` via Ports.",
        ]
    )


class FeatureEnvyStrategy(FixStrategy):
    """Strategy for resolving feature envy."""

    rule = "feature-envy"
    title = "Resolve Feature Envy"
    instructions = "\\n".join(
        [
            "Move logic closer to the data it operates on:",
            "- **Move Method**: If a method primarily uses another struct's data, move it there.",
            "- **Encapsulation**: Keep data and behavior together in `mcb-domain` entities.",
            "- **Getters**: If you are accessing many getters, it's a sign that logic belongs "
            "in that object.",
        ]
    )


class DataClumpStrategy(FixStrategy):
    """Strategy for encapsulating data clumps."""

    rule = "data-clump"
    title = "Encapsulate Data Clumps"
    instructions = "\\n".join(
        [
            "Group frequently appearing parameters or fields:",
            "- **Value Object**: Create a new struct in `mcb-domain::value_objects`.",
            "- **Validation**: Enforce invariants in the new type's constructor (`new()`).",
            "- **Type Safety**: Replace loose parameters with this strongly-typed value.",
        ]
    )


class BooleanLogicStrategy(FixStrategy):
    """Strategy for simplifying boolean logic."""

    rule = "boolean-logic"
    title = "Simplify boolean expressions"
    instructions = "\\n".join(
        [
            "Improve readability of boolean logic:",
            "- **Predicates**: Extract conditions into named methods returning `bool`.",
            "- **De Morgan**: Simplify negated groups.",
            "- **Matches**: Consider if a `match` expression is clearer than complex boolean "
            "operators.",
        ]
    )


class ComplexConditionStrategy(FixStrategy):
    """Strategy for simplifying complex conditions."""

    rule = "complex-condition"
    title = "Simplify complex conditional"
    instructions = BooleanLogicStrategy.instructions


class FunctionParametersStrategy(FixStrategy):
    """Strategy for reducing function parameters."""

    rule = "function-parameters"
    title = "Reduce function parameter count"
    instructions = "\\n".join(
        [
            "Too many arguments indicate missing abstractions:",
            "- **Config Struct**: Group related parameters into a configuration struct.",
            "- **Builder**: Use the Builder pattern for complex instance construction.",
            "- **Context**: Use a `Context` struct for passing cross-cutting data.",
        ]
    )


class TooManyArgumentsStrategy(FixStrategy):
    """Strategy for reducing too many arguments."""

    rule = "too-many-arguments"
    title = "Reduce argument count"
    instructions = FunctionParametersStrategy.instructions


class ReturnStatementsStrategy(FixStrategy):
    """Strategy for consolidating return statements."""

    rule = "return-statements"
    title = "Consolidate return points"
    instructions = "\\n".join(
        [
            "Simplify control flow exits:",
            "- **Expression-Oriented**: In Rust, the last expression is the return value. Use it.",
            "- **Guard Clauses**: Return early for error checks, then have a single success path.",
            "- **Result**: Propagate errors with `?` rather than manual early returns.",
        ]
    )


STRATEGIES = {
    s.rule: s()
    for s in (
        IdenticalCodeStrategy,
        SimilarCodeStrategy,
        FunctionComplexityStrategy,
        MethodComplexityStrategy,
        CognitiveComplexityStrategy,
        NestedControlFlowStrategy,
        DeepNestingStrategy,
        FileComplexityStrategy,
        LongMethodStrategy,
        LargeClassStrategy,
        GodClassStrategy,
        FeatureEnvyStrategy,
        DataClumpStrategy,
        BooleanLogicStrategy,
        ComplexConditionStrategy,
        FunctionParametersStrategy,
        TooManyArgumentsStrategy,
        ReturnStatementsStrategy,
    )
}


def get_strategy(rule_id: str) -> FixStrategy | None:
    """Get the appropriate fix strategy for a given rule ID."""
    # Rule ID might be "qlty:similar-code" or just "similar-code"
    short = rule_id.split(":")[-1]
    return STRATEGIES.get(short)
