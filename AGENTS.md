# Repository Notes for Agents

## Scenario descriptions

Keep player-facing puzzle descriptions focused on the objective and visible setup. Do not include intended solutions, packet timing tricks, exact exploit scripts, hidden validation details, or semantic/action-order hints in descriptions.

Do not add deterministic forbidden-word tests for natural-language description quality. Spoiler safety and setup plausibility must be reviewed semantically by agents/humans; deterministic tests should cover structural reveal gates only, such as not exposing `solution_script`, hidden lessons, or maintainer-only internal IDs in player-facing UI/API text.

For Arena 2, use the player-facing description:

- Arena 2
- Arena · ★★☆
- Kill the monster.

Put solution details only in implementation/reference fields such as `solution_script`, tests, or hidden author notes.
