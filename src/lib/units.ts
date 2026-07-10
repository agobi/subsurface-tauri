// AI-generated (Claude)
// Pure formatting functions for the five quantities Subsurface's units
// preference affects: depth, pressure, temperature, cylinder volume, weight.
// Conversion constants match Qt Subsurface exactly (core/units.h).
import type { Units } from "$lib/types.ts";

interface FmtOpts { suffix?: boolean; }

const M_TO_FT = 3.28084;       // mm_to_feet
const BAR_TO_PSI = 14.5037738; // mbar_to_PSI
const L_TO_CUFT = 28.3168466;  // ml_to_cuft (divide)
const KG_TO_LB = 2.2046226;    // grams_to_lbs
const ATM_BAR = 1.01325;       // bar_to_atm (divide)

export function fmtDepth(m: number, units: Units, opts?: FmtOpts): string {
  const suffix = opts?.suffix ?? true;
  if (units === "IMPERIAL") {
    const ft = Math.round(m * M_TO_FT);
    return suffix ? `${ft} ft` : `${ft}`;
  }
  const fixed = m.toFixed(1);
  return suffix ? `${fixed} m` : fixed;
}

export function fmtPressure(bar: number, units: Units, opts?: FmtOpts): string {
  const suffix = opts?.suffix ?? true;
  if (units === "IMPERIAL") {
    const psi = Math.round(bar * BAR_TO_PSI);
    return suffix ? `${psi} psi` : `${psi}`;
  }
  return suffix ? `${bar} bar` : `${bar}`;
}

export function fmtTemp(c: number, units: Units, opts?: FmtOpts): string {
  const suffix = opts?.suffix ?? true;
  if (units === "IMPERIAL") {
    const f = Math.round(c * 9 / 5 + 32);
    return suffix ? `${f} °F` : `${f}`;
  }
  return suffix ? `${c} °C` : `${c}`;
}

export function fmtVolume(l: number, units: Units, opts?: FmtOpts): string {
  const suffix = opts?.suffix ?? true;
  if (units === "IMPERIAL") {
    const cuft = (l / L_TO_CUFT).toFixed(2);
    return suffix ? `${cuft} cuft` : cuft;
  }
  return suffix ? `${l} L` : `${l}`;
}

// A cylinder's cuft rating is its physical volume times its working pressure
// in atmospheres (the "free gas at rated pressure" convention, e.g. an AL80
// is ~11.1 L but rated at ~77.4 cuft at 3000 psi) — not a plain L->cuft
// conversion of the tank's physical size. Matches Qt Subsurface's
// get_cylinder_string() (qt-models/cylindermodel.cpp), which for the same
// reason falls back to litres when the working pressure isn't known.
export function fmtCylinderSize(volumeL: number, workPressureBar: number | undefined, units: Units, opts?: FmtOpts): string {
  const suffix = opts?.suffix ?? true;
  if (units === "IMPERIAL" && workPressureBar != null) {
    const cuft = (volumeL / L_TO_CUFT) * (workPressureBar / ATM_BAR);
    const decimals = cuft > 20 ? 0 : cuft > 2 ? 1 : 2;
    const fixed = cuft.toFixed(decimals);
    return suffix ? `${fixed} cuft` : fixed;
  }
  return fmtVolume(volumeL, "METRIC", opts);
}

export function fmtWeight(kg: number, units: Units, opts?: FmtOpts): string {
  const suffix = opts?.suffix ?? true;
  if (units === "IMPERIAL") {
    const lb = Math.round(kg * KG_TO_LB);
    return suffix ? `${lb} lbs` : `${lb}`;
  }
  const fixed = kg.toFixed(2);
  return suffix ? `${fixed} kg` : fixed;
}
