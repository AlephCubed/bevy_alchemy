# Examples

| Example                               | Description                                                                    |
|---------------------------------------|--------------------------------------------------------------------------------|
| [`poison`](poison.rs)                 | A simple damage-over-time effect.                                              |
| [`poison_falloff`](poison_falloff.rs) | A damage-over-time effect where the damage falls off as more stacks are added. |

## Immediate Stats
Examples in the `immediate_stats` subdirectory utilize the [`immediate_stats`](https://github.com/AlephCubed/immediate_stats) crate, which I also created.
Some of these examples include a version that utilizes [`bevy_auto_plugin`](https://github.com/StrikeForceZero/bevy_auto_plugin), which should behave exactly the same.

| Example                                                                                                                              | Description                                                        |
|--------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------|
| [`decaying_speed`](immediate_stats/decaying_speed.rs), [`decaying_speed_auto_plugin`](immediate_stats/decaying_speed_auto_plugin.rs) | Movement speed that decreases in strength throughout its duration. |