## 1. Migration

- [x] 1.1 Add a reversible migration: deterministic dedupe of existing duplicate slugs (append `-N` suffixes), then `CREATE UNIQUE INDEX` on `channels.slug`; down migration drops the index
- [x] 1.2 Test migration on a seeded DB with a deliberately duplicated slug; assert dedupe + index creation and clean rollback

## 2. Conflict-Resilient Slug Creation

- [x] 2.1 Update `unique_slug`/insert path to retry with the next suffix on a DB unique-violation, preserving existing behavior for the normal path

> **Hallazgo de implementación (fix del retry):** la primera versión reintentaba con cualquier violación única, incluida la de `UNIQUE(channels.url)` al crear dos veces el mismo canal. Eso convertía el error de URL duplicada en un bucle de reintentos (~331K) generando suffixes absurdos (`atareao-331590`) y churn de DB. Corregido: el retry solo se dispara cuando la violación es específicamente de la constraint `channels.slug` (`is_slug_unique_violation`, discriminando por `channels.slug` en el mensaje del error). Una URL duplicada ahora falla al instante (500 con `UNIQUE constraint failed: channels.url`), igual que antes de este cambio.

## 3. Delete Ownership Guard

- [x] 3.1 Before `remove_dir_all`, verify no other channel row shares the slug directory; on conflict/suspicion log a warning and skip directory removal instead of wiping foreign files

## 4. Verification & Regression

- [x] 4.1 Create two channels with the same title (also concurrently) and assert distinct slugs; delete one and assert the other's files survive
- [x] 4.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #8 resolved and no new bugs introduced