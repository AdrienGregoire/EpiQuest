# Benchmark: Database Selection

Which database technology should be selected for EpiQuest at the start of the project? This document compares the available options to handle both highly structured relational data (users, campaigns, character sheets) and heterogeneous 5e SRD content (spells, monsters, features).

**In short:** **PostgreSQL** is selected. Its native `JSONB` support provides the flexibility required to import and query the SRD without sacrificing the relational integrity needed for users, campaigns, and combat action history.

## Comparative Overview

### Database Choice: Why PostgreSQL?

EpiQuest requires linking highly structured relational entities (users, campaigns) with complex, highly variable game data (5e SRD monsters, spells, and magic items).

| Criterion | PostgreSQL | MongoDB (NoSQL) | SQLite |
| :--- | :--- | :--- | :--- |
| **Relationship Management** (Players $\leftrightarrow$ Campaigns) | **Excellent** (foreign keys, integrity) | Complex | Good |
| **Data Flexibility** (5e SRD import, variable spells) | **Excellent** (via powerful `JSONB` type) | **Excellent** (document-oriented) | Basic |
| **Combat Reliability** (Logs & undo support) | **Excellent** (ACID transactions) | Good | Poor (write-locking issues) |
| **Verdict for the Project** | **Selected**: The ideal balance (combines the best of SQL and NoSQL). | **Discarded**: Overly complex for handling entity relationships. | **Discarded**: Cannot handle concurrent sessions in production. |

## Detailed Analysis of Technical Constraints

1. **Hybrid Data Model**:
   - *Strict Relational*: `User`, `Campaign`, `Character`, `Session`.
   - *JSONB Documents*: Spell mechanics, monster stat blocks, custom DM combat effects.
2. **Assisted Combat & Rollbacks (ACID)**:
   - Each combat action records state changes (HP, conditions, spent spell slots) in a transaction. In case of DM correction or undo, rollback must be immediate and atomic.
3. **Replayable 5e SRD Import**:
   - SRD data (from Open5e or JSON files) can be inserted directly into indexed `JSONB` columns without creating dozens of rigid relational tables.

## Recommendation

| Role | Selection | Reason |
| :--- | :--- | :--- |
| **Primary Database** | **PostgreSQL** | `JSONB` flexibility for SRD content combined with strong relational integrity and ACID transactions. |
| Development Setup | **Docker Compose** | Instant, identical local database provisioning for all team members. |