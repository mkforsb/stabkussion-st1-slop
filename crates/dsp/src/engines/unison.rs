//! SUPER: five detuned unison voices per oscillator.
//!
//! Detuned saws are the classic chord-stab thickener: the beating makes the
//! stab shimmer. Here every oscillator is a stack of five band-limited
//! saw/pulse voices, the outer pair detuned by DETUNE and the inner pair by an
//! irrational fraction of it, so the beating never lines up. The voices start
//! at fixed, spread-out phases: every hit has the same attack without the
//! stack slamming in phase.
//!
//! * DETUNE: outer voices ±0 … 80 cents.
//! * BLEND: level of the four side voices against the centre one.
//! * WAVE: saw → square.

use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{BlepOsc, Morph, Noise};

const VOICES: usize = 5;
/// Detune of each voice as a fraction of DETUNE.
const SPREAD: [f32; VOICES] = [-1.0, -0.43, 0.0, 0.47, 1.0];
/// Start phases (the centre voice starts at zero).
const PHASES: [f32; VOICES] = [0.37, 0.71, 0.0, 0.13, 0.58];

#[derive(Clone, Debug, Default)]
pub struct Unison {
    voices: [BlepOsc; VOICES],
    incs: [f32; VOICES],
    gains: [f32; VOICES],
    shape: Morph,
}

impl Osc for Unison {
    fn reset(&mut self) {
        for (v, &p) in self.voices.iter_mut().zip(&PHASES) {
            v.reset(p);
        }
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, _rng: &mut Noise) {
        self.reset();
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        let cents = map::unison_detune_cents(c.x);
        for (i, &s) in SPREAD.iter().enumerate() {
            self.incs[i] = inc(o.hz * (s * cents / 1200.0).exp2(), c.fs);
        }
        // Detuned, uncorrelated voices add up in power: normalize for that.
        let side = c.y;
        let norm = 1.0 / (1.0 + 4.0 * side * side).sqrt();
        for (i, g) in self.gains.iter_mut().enumerate() {
            *g = norm * if i == VOICES / 2 { 1.0 } else { side };
        }
        self.shape = Morph::saw_square(c.z);
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut y = 0.0;
        for i in 0..VOICES {
            if self.gains[i] != 0.0 {
                y += self.gains[i] * self.voices[i].tick(self.incs[i], &self.shape);
            }
        }
        y
    }
}
