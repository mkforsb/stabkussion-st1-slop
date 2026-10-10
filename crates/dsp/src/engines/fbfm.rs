//! FB-FM: a sine carrier phase-modulated by a sine with self-feedback.
//!
//! Feeding a modulator's output back into its own phase bends it from a sine
//! towards a saw (the DX7's operator feedback). Modulating with that instead
//! of a sine fills the gaps between the sidebands: at a whole-number ratio
//! the result is a dense, buzzy harmonic spectrum; at any other ratio it
//! is a thick inharmonic clang. The feedback path averages the last two
//! outputs, which keeps high feedback from breaking into noise.
//!
//! * RATIO: modulator/carrier ratio, 0.25 … 8, continuous.
//! * INDEX: modulation index, 0 … 6 rad; velocity scales it. It holds for
//!   the length of the note (the filter envelope does the closing).
//! * FDBK: modulator feedback, 0 … 1.4 rad.

use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{Noise, sin_turns};

#[derive(Clone, Debug, Default)]
pub struct FbFm {
    carrier: f32,
    modulator: f32,
    c_inc: f32,
    m_inc: f32,
    /// Index and feedback in turns.
    index: f32,
    feedback: f32,
    /// The modulator's last two outputs.
    y1: f32,
    y2: f32,
}

impl Osc for FbFm {
    fn reset(&mut self) {
        self.carrier = 0.0;
        self.modulator = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, _rng: &mut Noise) {
        self.reset();
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.c_inc = inc(o.hz, c.fs);
        self.m_inc = inc(o.hz * map::fm_ratio(c.x), c.fs);
        self.index = map::fbfm_index(c.y) * (0.4 + 0.6 * c.velocity) / core::f32::consts::TAU;
        self.feedback = map::fbfm_feedback(c.z) / core::f32::consts::TAU;
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let m = sin_turns(self.modulator + self.feedback * 0.5 * (self.y1 + self.y2));
        self.y2 = self.y1;
        self.y1 = m;
        let y = sin_turns(self.carrier + self.index * m);
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
