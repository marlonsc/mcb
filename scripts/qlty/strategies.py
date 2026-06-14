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
    instructions = "Refactor duplicated logic into shared abstractions:\\n- **Domain Logic**: Move shared business rules to `mcb-domain` entities or services.\\n- **Infrastructure**: Extract common technical implementations to `mcb-infrastructure::utils`.\\n- **Tests**: Use `mcb_domain::test_services_config` or shared test fixtures."


class SimilarCodeStrategy(FixStrategy):
    """Strategy for fixing similar code blocks."""

    rule = "similar-code"
    title = "Refactor similar code blocks"
    instructions = "Unify similar patterns using Rust's powerful type system:\\n- **Traits**: Define a trait in `mcb-domain::ports` and implement variations in `mcb-providers`.\\n- **Generics**: Use generic parameters for slight variations in types.\\n- **Macros**: Use `macro_rules!` (sparingly) for structural repetition that generics can't handle."


class FunctionComplexityStrategy(FixStrategy):
    """Strategy for reducing function complexity."""

    rule = "function-complexity"
    title = "Reduce function complexity"
    instructions = "Simplify complex functions by extracting logic:\\n- **Abstraction**: Move distinct steps into private helper methods or `impl` blocks.\\n- **Guard Clauses**: Use `if ... { return ... }` to reduce nesting depth.\\n- **Pattern Matching**: Use `match` expressions instead of complex `if/else` chains.\\n- **Error Handling**: Use the `?` operator for clean error propagation."


class MethodComplexityStrategy(FixStrategy):
    """Strategy for reducing method complexity."""

    rule = "method-complexity"
    title = "Reduce method complexity"
    instructions = FunctionComplexityStrategy.instructions


class CognitiveComplexityStrategy(FixStrategy):
    """Strategy for reducing cognitive complexity."""

    rule = "cognitive-complexity"
    title = "Lower cognitive complexity"
    instructions = "Make the code easier to reason about:\\n- **Encapsulation**: Hide complex details behind descriptive function names.\\n- **Boolean Logic**: Extract complex conditions into `is_valid()` styling methods.\\n- **Control Flow**: Prefer iterators (`map`, `filter`, `fold`) over manual loops with state."


class NestedControlFlowStrategy(FixStrategy):
    """Strategy for flattening nested control flow."""

    rule = "nested-control-flow"
    title = "Flatten deeply nested control flow"
    instructions = "Reduce nesting depth (target ≤ 4 levels):\\n- **Guard Clauses**: Check preconditions early and return.\\n- **Iterators**: Use functional combinators to transform collections flatly.\\n- **Lets**: Use `let ... = match ...` to assign results instead of nesting logic."


class DeepNestingStrategy(FixStrategy):
    """Strategy for flattening deep nesting."""

    rule = "deep-nesting"
    title = "Flatten deep nesting"
    instructions = NestedControlFlowStrategy.instructions


class FileComplexityStrategy(FixStrategy):
    """Strategy for splitting complex files."""

    rule = "file-complexity"
    title = "Split complex file into modules"
    instructions = "Break down large files into focused modules:\\n- **Modularity**: Create a directory with `mod.rs` and split concerns into separate files.\\n- **Clean Architecture**: Ensure the file strictly belongs to one layer (Domain, Infra, App).\\n- **Helpers**: Move utility functions to `utils.rs` or specialized submodules."


class LongMethodStrategy(FixStrategy):
    """Strategy for shortening long methods."""

    rule = "long-method"
    title = "Shorten long method"
    instructions = "Break methods into single-responsibility steps:\\n- **Steps**: Identify logical sections (setup, process, output) and extract them.\\n- **Size**: Aim for methods that fit on a single screen (≤ 25 lines).\\n- **Context**: If passing many variables, consider a context struct."


class LargeClassStrategy(FixStrategy):
    """Strategy for decomposing large classes."""

    rule = "large-class"
    title = "Decompose large struct/class"
    instructions = "Redistribute responsibilities from this large struct:\\n- **Composition**: Extract groups of fields into smaller Value Objects (in `mcb-domain::value_objects`).\\n- **Behavior**: Move complex logic to Domain Services if it involves multiple entities.\\n- **Traits**: Implement standard traits (`From`, `TryFrom`, `Display`) to offload conversion logic."


class GodClassStrategy(FixStrategy):
    """Strategy for decomposing god classes."""

    rule = "god-class"
    title = "Decompose God Class"
    instructions = "This struct violates Single Responsibility Principle:\\n- **Domain Services**: Split orchestration logic into specific Application Services.\\n- **Rich Entities**: Move business rules to the Entities that hold the data.\\n- **Providers**: Delegate external interaction to `mcb-providers` via Ports."


class FeatureEnvyStrategy(FixStrategy):
    """Strategy for resolving feature envy."""

    rule = "feature-envy"
    title = "Resolve Feature Envy"
    instructions = "Move logic closer to the data it operates on:\\n- **Move Method**: If a method primarily uses another struct's data, move it there.\\n- **Encapsulation**: Keep data and behavior together in `mcb-domain` entities.\\n- **Getters**: If you are accessing many getters, it's a sign that logic belongs in that object."


class DataClumpStrategy(FixStrategy):
    """Strategy for encapsulating data clumps."""

    rule = "data-clump"
    title = "Encapsulate Data Clumps"
    instructions = "Group frequently appearing parameters or fields:\\n- **Value Object**: Create a new struct in `mcb-domain::value_objects`.\\n- **Validation**: Enforce invariants in the new type's constructor (`new()`).\\n- **Type Safety**: Replace loose parameters with this strongly-typed value."


class BooleanLogicStrategy(FixStrategy):
    """Strategy for simplifying boolean logic."""

    rule = "boolean-logic"
    title = "Simplify boolean expressions"
    instructions = "Improve readability of boolean logic:\\n- **Predicates**: Extract conditions into named methods returning `bool`.\\n- **De Morgan**: Simplify negated groups.\\n- **Matches**: Consider if a `match` expression is clearer than complex boolean operators."


class ComplexConditionStrategy(FixStrategy):
    """Strategy for simplifying complex conditions."""

    rule = "complex-condition"
    title = "Simplify complex conditional"
    instructions = BooleanLogicStrategy.instructions


class FunctionParametersStrategy(FixStrategy):
    """Strategy for reducing function parameters."""

    rule = "function-parameters"
    title = "Reduce function parameter count"
    instructions = "Too many arguments indicate missing abstractions:\\n- **Config Struct**: Group related parameters into a configuration struct.\\n- **Builder**: Use the Builder pattern for complex instance construction.\\n- **Context**: Use a `Context` struct for passing cross-cutting data."


class TooManyArgumentsStrategy(FixStrategy):
    """Strategy for reducing too many arguments."""

    rule = "too-many-arguments"
    title = "Reduce argument count"
    instructions = FunctionParametersStrategy.instructions


class ReturnStatementsStrategy(FixStrategy):
    """Strategy for consolidating return statements."""

    rule = "return-statements"
    title = "Consolidate return points"
    instructions = "Simplify control flow exits:\\n- **Expression-Oriented**: In Rust, the last expression is the return value. Use it.\\n- **Guard Clauses**: Return early for error checks, then have a single success path.\\n- **Result**: Propagate errors with `?` rather than manual early returns."


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
