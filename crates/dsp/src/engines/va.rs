//! VA: a virtual-analog saw/pulse oscillator with pulse-width modulation.
//!
//! The saw and the pulse share one phase, so crossfading between them keeps
//! the edges lined up. The pulse's width moves smoothly (once per sample) and
//! its edge is found wherever it crosses the phase, so every jump, including
//! a moving one, is polyBLEP corrected. The pulse is DC-free at any width.
//!
//! Each oscillator gets its own slow PWM LFO, at a rate and phase drawn per
//! hit, so a chord of pulses drifts against itself like an analog poly.
//!
//! * WAVE: saw → pulse.
//! * WIDTH: pulse width, 50% (square) … 5%.
//! * PWM: depth of the slow width modulation.

use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{BlepOsc, Noise, sin_turns};
use crate::voice::CONTROL_BLOCK;

/// PWM LFO rates are drawn from this range (Hz).
const PWM_HZ: (f32, f32) = (0.3, 1.4);

#[derive(Clone, Debug, Default)]
pub struct Va {
    phase: f32,
    /// The polyBLEP output is one sample late (see [`BlepOsc`]).
    next: f32,
    inc: f32,
    saw: f32,
    pulse: f32,
    /// Current pulse width, and its per-sample step towards the target.
    width: f32,
    dw: f32,
    lfo: f32,
    lfo_hz: f32,
}

impl Va {
    /// The naive saw/pulse mix at phase `p` and width `w`. The saw rises
    /// through zero at phase 0 and jumps at ½ (like [`crate::util::Morph`]).
    #[inline]
    fn naive(&self, p: f32, w: f32) -> f32 {
        let saw = if p < 0.5 { 2.0 * p } else { 2.0 * p - 2.0 };
        let pulse = if p < w { 1.0 } else { -1.0 } - (2.0 * w - 1.0);
        self.saw * saw + self.pulse * pulse
    }
}

impl Osc for Va {
    fn reset(&mut self) {
        self.phase = 0.0;
        self.next = 0.0;
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.reset();
        self.lfo = rng.uniform();
        self.lfo_hz = PWM_HZ.0 + (PWM_HZ.1 - PWM_HZ.0) * rng.uniform();
        self.width = f32::NAN;
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.inc = inc(o.hz, c.fs);
        self.pulse = c.x;
        self.saw = 1.0 - c.x;
        self.lfo += self.lfo_hz * CONTROL_BLOCK as f32 / c.fs;
        if self.lfo >= 1.0 {
            self.lfo -= 1.0;
        }
        let w = map::pulse_width(c.y) + map::pwm_depth(c.z) * sin_turns(self.lfo);
        // Keep both halves at least two samples long.
        let edge = (2.0 * self.inc).clamp(0.03, 0.45);
        let target = w.clamp(edge, 1.0 - edge);
        if self.width.is_nan() {
            self.width = target;
            self.dw = 0.0;
        } else {
            self.dw = (target - self.width) / CONTROL_BLOCK as f32;
        }
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut this = self.next;
        let mut next = 0.0;
        let w0 = self.width;
        let w1 = w0 + self.dw;
        self.width = w1;
        let before = self.phase;
        let mut p = before + self.inc;
        // The pulse's falling edge, wherever the (moving) width put it.
        let (d0, d1) = (before - w0, p - w1);
        if self.pulse != 0.0 && d0 < 0.0 && d1 >= 0.0 {
            BlepOsc::blep(&mut this, &mut next, -2.0 * self.pulse, d1 / (d1 - d0));
        }
        if before < 0.5 && p >= 0.5 && self.saw != 0.0 {
            BlepOsc::blep(&mut this, &mut next, -2.0 * self.saw, (p - 0.5) / self.inc);
        }
        if p >= 1.0 {
            p -= 1.0;
            if self.pulse != 0.0 {
                BlepOsc::blep(&mut this, &mut next, 2.0 * self.pulse, p / self.inc);
            }
        }
        self.phase = p;
        self.next = next + self.naive(p, w1);
        this
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_has_no_dc_at_any_width() {
        let fs = 48_000.0;
        for y in [0.0, 0.5, 1.0] {
            let mut c = Ctx::new(fs);
            (c.x, c.y, c.z) = (1.0, y, 0.0);
            let o = OscCtx {
                hz: 100.0,
                env: 1.0,
                decay_s: 1.0,
            };
            let mut va = Va::default();
            va.trigger(&c, &o, &mut Noise::new(1));
            let n = 48_000;
            let mean = (0..n).map(|_| va.tick()).sum::<f32>() / n as f32;
            assert!(mean.abs() < 0.01, "width {y}: dc {mean}");
        }
    }
}
