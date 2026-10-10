//! FM: two-operator phase modulation, a sine modulating a sine.
//!
//! The modulation index has its own envelope, a faster decay than the
//! amplitude's, so a hit starts bright and metallic and relaxes into a pure
//! sine: the classic FM bass pluck and the "DX kick". Velocity scales the
//! index.
//!
//! * RATIO: modulator/carrier frequency ratio, 0.25 … 8, continuous. Whole
//!   numbers are harmonic, everything else clangs.
//! * INDEX: peak modulation index, 0 … 12 radians.
//! * MOD DEC: index decay as a multiple of the oscillator's decay (×0.03 …
//!   ×1, where the brightness lasts as long as the sound).

use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{Noise, sin_turns, t60_gain};

#[derive(Clone, Debug, Default)]
pub struct Fm {
    carrier: f32,
    modulator: f32,
    c_inc: f32,
    m_inc: f32,
    /// Peak index in turns.
    index: f32,
    index_env: f32,
    index_mul: f32,
}

impl Osc for Fm {
    fn reset(&mut self) {
        self.carrier = 0.0;
        self.modulator = 0.0;
        self.index_env = 0.0;
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, _rng: &mut Noise) {
        self.reset();
        self.index_env = 1.0;
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.c_inc = inc(o.hz, c.fs);
        self.m_inc = inc(o.hz * map::fm_ratio(c.x), c.fs);
        self.index = map::fm_index(c.y) * (0.4 + 0.6 * c.velocity) / core::f32::consts::TAU;
        self.index_mul = t60_gain(o.decay_s * map::fm_index_decay(c.z), c.fs);
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let m = self.index * self.index_env * sin_turns(self.modulator);
        self.index_env *= self.index_mul;
        let y = sin_turns(self.carrier + m);
        self.carrier += self.c_inc;
        if self.carrier >= 1.0 {
            self.carrier -= 1.0;
        }
        self.modulator += self.m_inc;
        if self.modulator >= 1.0 {
            self.modulator -= 1.0;
        }
        y
    }
}
