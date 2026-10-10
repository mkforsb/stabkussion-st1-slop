//! SYNC: a virtual-analog oscillator hard-synced to a hidden master.
//!
//! The master runs at the oscillator's pitch and only resets the slave, so the
//! pitch you hear is always the master's; the slave's frequency, a multiple of
//! it, moves a formant-like peak through the spectrum. All jumps (the slave's
//! own and the sync resets) are polyBLEP corrected.
//!
//! * WAVE: the slave's shape, sine → triangle → saw → square.
//! * SYNC: static slave ratio, ×1 (no sync) … ×8.
//! * SWEEP: how many octaves the ratio rises at the envelope peak: the classic
//!   sync "zap" that settles as the hit decays. Velocity scales it.
//!
//! Hard sync moves energy away from the fundamental as the ratio rises, which
//! thins a bass out. So the master is audible too: a sine at the pitch is
//! blended in as the ratio climbs (up to half the mix), keeping the note
//! anchored under the zap.

use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{BlepOsc, Morph, Noise, sin_turns};

#[derive(Clone, Debug, Default)]
pub struct Sync {
    master: f32,
    master_inc: f32,
    slave: BlepOsc,
    slave_inc: f32,
    shape: Morph,
    /// Share of the master's sine in the output.
    anchor: f32,
    /// Master phase one sample back, matching the slave's delayed output.
    last_master: f32,
}

impl Osc for Sync {
    fn reset(&mut self) {
        self.master = 0.0;
        self.last_master = 0.0;
        self.slave.reset(0.0);
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, _rng: &mut Noise) {
        self.reset();
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.master_inc = inc(o.hz, c.fs);
        let sweep = map::sync_sweep_octaves(c.z) * o.env * (0.5 + 0.5 * c.velocity);
        let ratio = map::sync_ratio(c.y) * sweep.exp2();
        self.slave_inc = inc(o.hz * ratio, c.fs);
        self.shape = Morph::sweep(c.x);
        self.anchor = 0.5 * (1.0 - 1.0 / ratio);
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let fundamental = sin_turns(self.last_master);
        self.last_master = self.master;
        self.master += self.master_inc;
        let slave = if self.master >= 1.0 {
            self.master -= 1.0;
            let t = self.master / self.master_inc.max(1e-9);
            self.slave
                .tick_synced(self.slave_inc, &self.shape, t.min(1.0))
        } else {
            self.slave.tick(self.slave_inc, &self.shape)
        };
        slave + (fundamental - slave) * self.anchor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(x: f32, y: f32, z: f32, hz: f32, n: usize) -> Vec<f32> {
        let mut c = Ctx::new(96_000.0);
        (c.x, c.y, c.z) = (x, y, z);
        let o = OscCtx {
            hz,
            env: 1.0,
            decay_s: 1.0,
        };
        let mut s = Sync::default();
        s.trigger(&c, &o, &mut Noise::new(1));
        (0..n).map(|_| s.tick()).collect()
    }

    #[test]
    fn unsynced_sine_is_a_clean_sine_at_pitch() {
        let out = render(0.0, 0.0, 0.0, 100.0, 96_000);
        let crossings = out.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count();
        assert!((crossings as i32 - 100).abs() <= 1, "{crossings}");
        let peak = out.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!((peak - 1.0).abs() < 0.01);
    }

    #[test]
    fn synced_output_repeats_at_the_master_pitch() {
        // Autocorrelation at the master period beats the slave period.
        let out = render(0.66, 0.6, 0.0, 110.0, 19_200);
        let period = (96_000.0f32 / 110.0).round() as usize;
        let corr = |lag: usize| {
            (2_000..10_000).map(|i| out[i] * out[i + lag]).sum::<f32>()
                / (2_000..10_000).map(|i| out[i] * out[i]).sum::<f32>()
        };
        assert!(corr(period) > 0.95, "{}", corr(period));
        assert!(out.iter().all(|s| s.is_finite() && s.abs() < 1.6));
    }
}
