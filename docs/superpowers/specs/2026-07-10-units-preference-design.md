# Honor units preference for display (#101)

## Problem

The logbook's `units` header (METRIC/IMPERIAL) is parsed and round-tripped
through `Logbook.units`, but nothing in the UI reads it — every component
hardcodes metric display (depth in m, pressure in bar, temp in °C, cylinder
size in L, weight in kg). Scope is frontend display/formatting only: stored
values and the git logbook format stay metric-only (matching Qt Subsurface),
we're only adding a conversion+formatting layer for the UI.

## Scope of converted fields

Matches Qt Subsurface's own scope (`core/units.h`/`units.cpp`), which bundles
one imperial/metric flag across five quantities: **depth, pressure,
temperature, cylinder volume, and total weight.**

Explicitly **out of scope**: the DiveList "SAC" column. It's a derived ratio
with no stored raw field, and today shows no unit label at all — converting
it is a separate, non-trivial change not requested by this issue.

## Conversion constants (matched to Qt Subsurface exactly)

From `core/units.h`:

| Quantity | Formula | Source |
|---|---|---|
| depth | `ft = m * 3.28084` | `mm_to_feet` |
| pressure | `psi = bar * 14.5037738` | `mbar_to_PSI` |
| temperature | `F = C * 9/5 + 32` | `mkelvin_to_F` |
| volume | `cuft = L / 28.3168466` | `ml_to_cuft` |
| weight | `lb = kg * 2.2046226` (i.e. `grams/453.6`) | `grams_to_lbs` |

## Precision

**Metric formatting is byte-for-byte unchanged from today** — no rounding
behavior is added to the existing default path. Imperial precision is new
and matches Qt (and the "whole numbers" choice already made for this
project):

| Quantity | Metric (unchanged) | Imperial (new) |
|---|---|---|
| depth | `.toFixed(1)` + `" m"` | `Math.round()` + `" ft"` |
| pressure | raw passthrough, no suffix today | `Math.round()` + `" psi"` |
| temperature | raw passthrough | `Math.round()` + `" °F"` |
| volume | raw passthrough | `.toFixed(2)` + `" cuft"` (matches Qt) |
| weight | `.toFixed(2)` (existing) | `Math.round()` + `" lbs"` (matches Qt) |

## Architecture

### 1. Formatting layer — `src/lib/units.ts` (new)

Pure functions, one per quantity, each `(value: number, units: Units, opts?: { suffix?: boolean }) => string`:

```ts
fmtDepth(m, units, opts?)
fmtPressure(bar, units, opts?)
fmtTemp(c, units, opts?)
fmtVolume(l, units, opts?)
fmtWeight(kg, units, opts?)
```

`suffix` defaults to `true` (returns e.g. `"49 ft"`); `suffix: false` returns
just the converted/rounded number as a string, for table cells where the
unit lives in the column header instead. `Units` is the existing
`"METRIC" | "IMPERIAL"` type from `types.ts` — no new type needed for the
domain value itself.

### 2. Preference resolution — mirrors the existing Theme pattern exactly

- New `UnitsPref = "auto" | "METRIC" | "IMPERIAL"` in `stores/app.svelte.ts`
  (parallel to `Theme`). `"auto"` (default) follows `Logbook.units`;
  explicit `METRIC`/`IMPERIAL` overrides it.
- `AppearancePrefs` (`prefs.ts`) gains a `units: UnitsPref` field, persisted
  in the same `settings.json` "appearance" bucket as theme. New pure helper
  `resolveUnits(pref: UnitsPref, logbookUnits: Units): Units`, parallel to
  `resolveTheme`. `loadAppearancePrefs()` merges saved data over
  `{ theme: "auto", units: "auto" }` defaults for backward compatibility
  with settings.json files saved before this change.
- `AppStore` gains `unitsPref = $state<UnitsPref>("auto")`,
  `setUnitsPref(u)`, and a `get displayUnits(): Units` getter delegating to
  `resolveUnits(this.unitsPref, this.logbook.units)`. `reset()` resets
  `unitsPref` to `"auto"`.
- UI: a new "Units" radio group (Auto/Metric/Imperial) added to
  `AppearanceSection.svelte` next to the existing Theme radio group, same
  component reused on both the desktop prefs window (`Prefs.svelte` →
  `PrefsShell.svelte`) and the Android inline settings screen
  (`MobileSettingsScreen.svelte`) — identical dual-path wiring the Theme
  preference already uses, including the `prefs:appearance-changed` event
  so a change in the prefs window updates the live main window.

### 3. Call sites — two conventions, matched to what's already there

**Single-value displays** (full string with suffix):
- `DiveProfile.svelte`: cursor tooltip depth (`fmtDepth`), axis tick labels
  (see below).
- `info/InformationTab.svelte`: Max depth, Mean depth, Water temp rows.
- `DcDownloadDialog.svelte`: per-row max depth in the review list.

**Tabular columns** (numeric-only cells, unit named once in the header):
- `diveListColumns.ts`: `depth`, `temp`, `weight` columns. `RenderCtx` gains
  a `units: Units` field (threaded from `DiveList.svelte` as
  `app.displayUnits`); only `render` functions change — `compare` functions
  are untouched since unit conversion is monotonic and doesn't affect sort
  order. Column header labels (`ColDef.label`) stay static/unit-agnostic,
  matching current behavior (they show no unit today either).
- `info/EquipmentTab.svelte`: cylinder table (Size/Work/Start/End columns).
  Unlike DiveList, this table currently shows *no* unit indicator anywhere,
  so headers become unit-aware dynamically (e.g. `Size (L)` / `Size (cuft)`,
  `Work (bar)` / `Work (psi)`) so the change in numbers is legible.

All non-DiveList/EquipmentTab components read `app.displayUnits` directly
via the existing `app` store singleton import (same pattern `DiveList.svelte`
already uses) — no prop-threading needed through `InfoPanel.svelte`.

### 4. Dive profile depth-axis gridlines — `profile-scale.ts`

Real Subsurface computes tick intervals adaptively from available chart
height, snapping to "nice" numbers computed **natively in the currently
displayed unit** (never by converting the other unit's nice number) —
confirmed by reading `profile-widget/divecartesianaxis.cpp`. Porting that
full adaptive algorithm (plus its separate opt-in "3m/nice-multiples-of-3
grid" preference, `qPrefDisplay::three_m_based_grid`, default off) is out of
scope for this issue.

This project's existing prototype already uses a simplified **fixed** step
(5 m) rather than the adaptive algorithm. Per diving-conventions feedback,
that fixed step becomes:

- **Metric: 6 m** (changed from the current 5 m — a more realistic
  dive-instrument increment, and it also matches Subsurface's
  multiples-of-three "nice" set).
- **Imperial: 10 ft** (new).

Changes:
- `depthAxisMax(maxDepthM: number): number` — ceiling step changes from 5 to
  6: `Math.ceil((maxDepthM + 4) / 6) * 6`. Always computed in metres (sample
  data is always metric) — this is the Y-scale bound, independent of display
  unit. **This changes existing behavior**: `depthAxisMax(34.7)` goes from
  `40` → `42`, `depthAxisMax(18.2)` goes from `25` → `24`. The existing test
  in `test/profile/profile-scale.test.ts` will be updated to match.
- New `depthGridLines(axisMaxM: number, units: Units): { m: number; label: number }[]`
  — returns tick positions in **metres** (`m`, fed into the existing
  `depthToY` for correct proportional placement against the always-metric
  sample data) paired with a **display-unit label** (`label`): every 6 m
  (label = same value) for metric, every 10 ft converted to metres for
  positioning (label = `Math.round(ft)`) for imperial. `DiveProfile.svelte`
  renders `{tick.label}{units === "IMPERIAL" ? "ft" : "m"}` instead of the
  current inline `gridLines()` + `{m}m` template — this local ternary is
  the only place a bare unit abbreviation is needed; it's not exported from
  `units.ts`, which only exposes the full `fmtX()` formatters.
- The tank-pressure/temperature overlay polylines' normalization caps (250
  bar, 40°C) are unaffected — they scale off raw metric sample values only
  for line positioning, never displayed as text.

## Testing

- `test/lib/units.test.ts` (new): conversion + formatting correctness for
  all 5 quantities, both units, rounding edge cases, `suffix: false`.
- `test/profile/profile-scale.test.ts`: update `depthAxisMax` expectations
  for the 6 m step; add `depthGridLines` cases for both units.
- `test/lib/diveListColumns.test.ts`: extend `RenderCtx` fixtures with
  `units`; add imperial-branch assertions for depth/temp/weight `render`.
- `test/lib/prefs.test.ts`: `resolveUnits` cases; `loadAppearancePrefs`
  default-merge with a settings.json missing the `units` key (back-compat).
- `test/stores/app.test.ts`: `displayUnits` getter (auto/explicit),
  `setUnitsPref`, `reset()`.
- `test/components/prefs/AppearanceSection.test.ts`: Units radio group
  render + change callback.
- `test/components/DiveProfile.test.ts`, `test/DcDownloadDialog.test.ts`,
  `test/components/InfoPanel.test.ts`: imperial-mode assertions alongside
  existing metric ones.
- `test/components/App.desktop.test.ts` / `App.mobile.test.ts`,
  `test/components/MobileSettingsScreen.test.ts`: load/listen/wire the new
  `units` field through the same paths already tested for `theme`.
- Visual regression: depth-axis gridline count/spacing changes (5m→6m step)
  will shift `DiveProfile` snapshot pixels — baselines will need updating
  via the existing `update-snapshots` GitHub Actions workflow.
