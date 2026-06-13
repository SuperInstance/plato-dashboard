# DEPENDENCIES — plato-dashboard

## Signal Chain Layer

**Cross-cutting (Visualization)** — Fleet dashboard rendering.

Fleet dashboard for PLATO nervous system. Renders room states, autonomy metrics, signal chain throughput, and fleet coordination status.

## Ecosystem Dependencies

| Repo | Relationship | Description |
|------|-------------|-------------|
| [plato-state](https://github.com/SuperInstance/plato-state) | **Depends on** | Room state vectors for rendering room status |
| [plato-autonomy](https://github.com/SuperInstance/plato-autonomy) | **Depends on** | Autonomy metrics for fleet health visualization |
| [plato-coordination](https://github.com/SuperInstance/plato-coordination) | **Related** | Fleet coordination status for cross-room views |
| [plato-nervous](https://github.com/SuperInstance/plato-nervous) | **Related** | Signal chain metrics for throughput visualization |

## Data Flow

```
IN:
  - Room state vectors (from plato-state)
  - Autonomy scores (from plato-autonomy)
  - Fleet coordination status
  - Signal chain metrics

OUT:
  - Fleet dashboard rendering (HTML/terminal/other)
  - Room status cards
  - Fleet health summary
  - Alert visualizations
```

## Dependency Graph Position

```
plato-tiles → plato-rooms → plato-state → plato-autonomy
                              ↓                  ↓
                    plato-dashboard ← (this crate)
```
