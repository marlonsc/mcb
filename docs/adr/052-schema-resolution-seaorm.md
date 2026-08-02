<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# ADR 052: Schema Resolution with SeaORM 2.x

## Status

Accepted

## Context

The current persistence architecture already has a canonical schema model in
`mcb-domain`:

- `Schema` aggregates tables, indexes, FKs, unique constraints, and FTS
  definitions.
- `TableDef` and `ColumnDef` express backend-agnostic structure with logical
  types (`Text`, `Integer`, `Json`, `Uuid`, etc.).
- `SchemaDdlGenerator` is a port that emits backend-specific DDL from the same
  canonical definition.

In `mcb-providers`, `SqliteSchemaDdlGenerator` consumes this domain schema and
materializes SQLite DDL, including composite PK handling, FK clauses, indexes,
and FTS trigger rebuild.

SeaORM 2.x entities serve a different purpose. `EntityTrait` and `ColumnTrait`
define ORM metadata plus runtime query semantics (find/insert/update/delete,
filters, relation loading, and expression building). SeaORM also supports an
Entity First workflow that can derive schema changes from entity definitions.

This creates an architectural choice:

1. Replace the domain schema and let SeaORM entities become the sole source of
   schema truth.
2. Keep the domain schema as a specification and treat SeaORM entities as an
   implementation detail in provider adapters.

Constraints:

- Clean Architecture dependency flow must remain inward.
- Domain must not depend on provider/ORM frameworks.
- `make validate` enforces architecture boundaries and should remain green
  without adding framework coupling exceptions.

## Decision

Keep the domain schema (`Schema`/`TableDef`/`SchemaDdlGenerator`) as the
canonical specification. Use SeaORM entities as provider-layer implementation,
not as domain-level schema authority.

Decision detail:

- `mcb-domain` continues to define storage intent in framework-agnostic terms.
- `mcb-providers` may implement SeaORM entities for persistence operations,
  migrations, and adapter-level query ergonomics.
- Any SeaORM entity generated/maintained schema must be validated against the
  domain schema contract, not vice versa.
- Schema evolution decisions are recorded at the domain specification level;
  provider models follow that contract.

## Consequences

### Positive Consequences

- Preserves Clean Architecture: no SeaORM dependency leaks into `mcb-domain`.
- Maintains an explicit specification/implementation split, which keeps adapters
  swappable and testable.
- Protects portability if additional database providers or non-SeaORM adapters
  are introduced.
- Keeps `make validate` unchanged: existing boundary rules continue to pass
  without introducing domain->provider exceptions.
- Avoids coupling domain contracts to ORM query APIs (`EntityTrait`/
  `ColumnTrait`) that are operational rather than canonical specification.

### Negative Consequences

- Two schema representations must stay aligned (domain spec and SeaORM entity
  view).
- Requires explicit mapping/conformance checks between domain definitions and
  provider entities.
- Adds short-term overhead during v0.3.0 migration work (entities and
  conversions).

### Neutral Consequences

- No immediate code changes in this ADR; implementation is deferred to Wave 1.
- Existing SQLite DDL generation remains valid and continues to use domain
  schema as input.

## Alternatives Considered

### Alternative 1: Replace domain schema with SeaORM entities

- **Pros**: One model for DDL + CRUD, less apparent duplication.
- **Cons**: Pushes ORM concerns into architecture center, weakens domain
  isolation, and risks coupling validation rules to one framework.
- **Rejection Reason**: Conflicts with Clean Architecture intent and creates
  long-term framework lock-in in the core model.

### Alternative 2: Keep both as co-equal sources of truth

- **Pros**: Incremental migration flexibility.
- **Cons**: Ambiguous ownership causes drift and decision conflicts.
- **Rejection Reason**: Governance ambiguity increases maintenance risk; one
  canonical owner is required.

## References

- `crates/mcb-domain/src/schema/types.rs`
- `crates/mcb-domain/src/schema/definition.rs`
- `crates/mcb-providers/src/database/sqlite/ddl.rs`
- SeaORM 2.x docs: <https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-format/>
- SeaORM API docs (`EntityTrait`, `ColumnTrait`):
  <https://docs.rs/sea-orm/latest/sea_orm/entity/trait.EntityTrait.html>,
  <https://docs.rs/sea-orm/latest/sea_orm/entity/trait.ColumnTrait.html>
