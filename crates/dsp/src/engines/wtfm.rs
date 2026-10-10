//! WT-FM: a wavetable carrier phase-modulated by a sine.
//!
//! The wavetable gives the body its colour, the FM gives it a bark. As in FM,
//! the index decays faster than the amplitude (a third of the oscillator's
//! decay), so the hit snarls and then settles into the plain table.
//!
//! * POS: carrier table position (as on TABLE).
//! * INDEX: peak modulation index, 0 … 6 radians; velocity scales it.
//! * RATIO: modulator ratio, 0.25 … 8, continuous.

use super::tables::{FRAMES, Mip, Tables, tables};
use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{Noise, sin_turns, t60_gain, wrap};

const INDEX_DECAY_RATIO: f32 = 0.33;

#[derive(Clone, Debug, Default)]
pub struct WtFm {
    carrier: f32,
    modulator: f32,
    c_inc: f32,
    m_inc: f32,
    pos: f32,
    mip: Mip,
    /// Peak index in turns.
    index: f32,
    index_env: f32,
    index_mul: f32,
}

impl Osc for WtFm {
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
        let ratio = map::fm_ratio(c.z);
        self.m_inc = inc(o.hz * ratio, c.fs);
        self.pos = c.x * (FRAMES - 1) as f32;
        let index = map::wtfm_index(c.y) * (0.4 + 0.6 * c.velocity);
        self.index = index / core::f32::consts::TAU;
        self.index_mul = t60_gain(o.decay_s * INDEX_DECAY_RATIO, c.fs);
        // Shifting the phase of a whole table shifts harmonic k's phase k
        // times as far: its peak frequency is k·(carrier + index·modulator),
        // plus one modulator frequency of Carson sideband. Pick the mip for
        // that. As the index decays the table opens up again, crossfading
        // between levels (see `Tables::mip_blend`) so nothing clicks.
        let beta = index * self.index_env;
        let fm = o.hz * ratio;
        let (swing, sidebands) = if beta > 0.01 {
            (beta * fm, fm)
        } else {
            (0.0, 0.0)
        };
        self.mip = Tables::mip_blend(o.hz + swing, sidebands, c.fs);
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let m = self.index * self.index_env * sin_turns(self.modulator);
        self.index_env *= self.index_mul;
        let y = tables().read_mip(self.pos, self.mip, wrap(self.carrier + m));
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
