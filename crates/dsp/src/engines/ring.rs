//! RING: a virtual-analog wave ring-modulated by a sine.
//!
//! Multiplying by a sine at `r` times the pitch moves every harmonic `k` of
//! the wave to `k ± r`: unless `r` is a whole number (or a half) the result
//! is inharmonic, the bell-like, detuned clang of a ring-modulated stab. The
//! wave is the polyBLEP morph SYNC uses, so its edges stay clean; the product
//! only reaches `r` pitches higher, well within the oversampled band.
//!
//! * WAVE: sine → triangle → saw → square.
//! * RATIO: ring modulator ratio, 0.25 … 8, continuous.
//! * RING: amount, from the plain wave (0) to full ring modulation (1).

use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{BlepOsc, Morph, Noise, sin_turns};

#[derive(Clone, Debug, Default)]
pub struct Ring {
    wave: BlepOsc,
    w_inc: f32,
    shape: Morph,
    /// The modulator phase one sample back, to line up with the polyBLEP
    /// oscillator's one-sample delay.
    r_last: f32,
    r_phase: f32,
    r_inc: f32,
    amount: f32,
}

impl Osc for Ring {
    fn reset(&mut self) {
        self.wave.reset(0.0);
        self.r_phase = 0.0;
        self.r_last = 0.0;
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, _rng: &mut Noise) {
        self.reset();
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.w_inc = inc(o.hz, c.fs);
        self.shape = Morph::sweep(c.x);
        self.r_inc = inc(o.hz * map::fm_ratio(c.y), c.fs);
        self.amount = c.z;
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let w = self.wave.tick(self.w_inc, &self.shape);
        // A cosine, so the hit starts at full level rather than in a null of
        // the modulation.
        let ring = sin_turns(self.r_last + 0.25);
        self.r_last = self.r_phase;
        self.r_phase += self.r_inc;
        if self.r_phase >= 1.0 {
            self.r_phase -= 1.0;
        }
        w * (1.0 - self.amount + self.amount * ring)
    }
}
