# Serde Quick Reference

## Use in This Repo

- config parsing and serialization
- API payload models
- persistence model encoding/decoding

## Preferred Patterns

- derive `Serialize`/`Deserialize` on stable data models
- use explicit renames when wire formats differ
- avoid silent defaults that hide migration issues

## Validation Reminders

- keep backward-compatible field evolution where required
- validate optional fields at boundaries
- test representative payload round-trips

> Updated 2026-02-12
