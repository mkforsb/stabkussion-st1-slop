# Stabkussion ST-1

A two-channel **techno stab** synthesizer: dub chords, FM clangs, gated
minor sevenths, ring-modulated hits and filtered plucks, mostly bassy and
sometimes inharmonic. It is a sibling of
[Subkussion SB-2](https://github.com/mkforsb/subkussion-sb2-slop) and keeps
its frame: two channels, a trigger pad per channel, the ladder filter,
saturator, EQ, chorus and plate reverb, MUTATE and RND, and locks on every
control that RND can change. What changes is the voice. Every channel runs
**four freely tuned oscillators** of one of eight engines (virtual analog,
hard sync, unison, FM, feedback FM, ring modulation, phase distortion and
wavetable FM), through an amplitude envelope with a gate, a filter
envelope of its own, and a ping-pong **echo** in front of the plate.

ST-1 was built against a set of 27 reference stabs. A fitting tool searches
the panel for each one, and RND is shaped (and tested) so that its hits
land among them. See [How close it gets](#how-close-it-gets).

It's written in Rust with a [Dioxus](https://dioxuslabs.com) UI and builds for
two targets from one codebase:

| Target | Audio | How |
| --- | --- | --- |
| Web (wasm) | WebAudio `AudioWorklet` | `make serve-web` |
| Native Linux | PulseAudio (also PipeWire's pulse server) | `make run-desktop` |

## Building

Requirements:

- Rust (stable) with `rustup target add wasm32-unknown-unknown`
- Dioxus CLI 0.7: `cargo binstall dioxus-cli` or grab `dx` from the Dioxus releases
- Desktop only (Debian/Ubuntu package names): `libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libssl-dev libpulse-dev pkg-config`

```sh
make serve-web      # http://127.0.0.1:8080, click once to enable audio
make run-desktop    # native window
make web desktop    # release bundles in target/dx/stabkussion/release/
make test lint
make renders        # offline-render every preset to ./renders/*.wav
make bench          # engine speed as a real-time factor
make calibrate      # loudness survey of every engine
make stabs          # how stab-like RND is, against the WAVs in ./examples
make fit            # fit a patch to every WAV in ./examples (slow)
```

For the desktop build, `STABKUSSION_LATENCY_MS` (default 20) sets the
PulseAudio target latency.

## Playing it

- **Pads**: click a channel's STRIKE pad. Where you click sets the velocity:
  near the top is a hard hit, near the bottom a soft one. Velocity sets the
  level, how far the filter envelope opens, and on most engines the
  brightness too (FM index, sync sweep, distortion).
- **Keyboard**: `Z X C V` trigger channel 1 at increasing velocity,
  `N M , .` do the same for channel 2, and `Space` triggers both.
- **Knobs/faders**: drag vertically. Hold `Shift` for fine control, use the
  scroll wheel to nudge, and double-click to reset.
- **Locks**: **Ctrl-click** (or ⌘-click) any knob, fader, switch or engine
  button to lock it. A locked control shows a padlock and turns rose, and
  RND and MUTATE leave it exactly where it is. Ctrl-click again to unlock.
  The padlock counter in the channel header shows how many controls are
  locked; click it to unlock them all. Locks belong to the channel, not the
  patch: loading a preset or copying a channel changes locked values but
  keeps them locked. They are saved with the rest of the panel.
  - Everything that RND can change is lockable. OUTPUT, SENSE and PAN are
    never randomized, so they have no lock.
  - Locking never changes what the free controls roll. Every control still
    draws its own random number, in a fixed order, whether or not it's
    locked.
  - The rules RND applies afterwards (below) only move free controls.
- **MUTATE** (the knob under each pad) changes the channel's sound on every
  hit. Each trigger draws a fresh random patch, as RND would, and moves each
  free control that fraction of the way towards it:
  `clamp(current + (random − current) × MUTATE)`. At **RND** every hit is a
  new stab. Lock PITCH and the four TUNEs, set MUTATE to 30% and the chord
  stays put while its timbre wanders.
- **Presets**, **RND** and **→ CHn** (copy a channel to the other) are in
  each channel's header. The whole panel is saved automatically: to
  `localStorage` on the web, and to `~/.config/stabkussion-st1/state.txt` on
  the desktop.
- The **HIT** display is an offline render of one hit of the current patch,
  and the **EQ** display is the EQ's actual response, computed by the same
  filter code the voice runs.

## The oscillators

Every engine is an oscillator type. A channel runs four of them, **OSC 1–4**:

- **PITCH** (16 Hz – 1 kHz) and **FINE** tune OSC 1. PITCH is continuous, not
  quantized to notes.
- **TUNE** sets OSC 2, 3 and 4 relative to OSC 1, ±24 semitones,
  continuous. Shift-drag for hundredths of a semitone. A minor seventh
  stab is +3, +7, +10; just intervals (+7.02 for 3:2), slightly sour
  chords, octave doublings a few cents off and inharmonic clusters are
  all a turn away.
- **LEVEL** sets each oscillator's level. Stacked oscillators are scaled back
  towards the loudness of one.
- **DECAY** for OSC 2–4 is a multiple of the main DECAY (×0.06 … ×2): a
  chord tone that dies before the root, or one that outlasts it.
- All of them share the envelope and follow the **SWEEP**, the chaos
  modulator and the per-hit **JITTER**.

## The engines

The three big knobs under the faders are the engine's macros, shared by all
four oscillators. They are relabelled for each engine, and so is the chaos
modulator's DEST switch.

| Engine | Model | X | Y | Z |
| --- | --- | --- | --- | --- |
| **VA** | Virtual analog: polyBLEP saw and pulse on one phase. Each oscillator gets its own slow PWM LFO, at a rate and phase drawn per hit | WAVE: saw → pulse | WIDTH: 50% … 5% | PWM: depth |
| **SYNC** | polyBLEP sine/tri/saw/square hard-synced to a hidden master, whose sine is blended back in as the ratio rises | WAVE | SYNC: ×1 … ×8 | SWEEP: extra octaves of ratio at the envelope peak |
| **SUPER** | Five detuned polyBLEP voices per oscillator | DETUNE: ±0 … 80 cents | BLEND: side voices vs centre | WAVE: saw → square |
| **FM** | Two-operator FM with its own index envelope | RATIO: 0.25 … 8, continuous | INDEX: 0 … 12 rad | MOD DEC: index decay ×0.03 … ×1 |
| **FB-FM** | A sine carrier modulated by a sine with self-feedback (DX-style), which bends the modulator towards a saw | RATIO: 0.25 … 8 | INDEX: 0 … 6 rad | FDBK: 0 … 1.4 rad |
| **RING** | A polyBLEP wave times a sine at a free ratio: every harmonic *k* moves to *k ± r* | WAVE | RATIO: 0.25 … 8 | RING: dry wave … full ring modulation |
| **PD** | Casio CZ-style phase distortion | DIST | WAVE: saw → square → reso → pulse | D.ENV: how much of DIST follows the envelope |
| **WT-FM** | A band-limited wavetable (SINE … SAW, FAT) phase-modulated by a sine whose index decays | POS | INDEX: 0 … 6 rad | RATIO: 0.25 … 8 |

FM, FB-FM and RING at whole-number ratios are harmonic; anywhere else they
are the inharmonic, metallic stabs. Velocity scales the FM indices.

### Anti-aliasing

The voices run at 2× the output rate and a 47-tap windowed-sinc filter
decimates each one, so a partial between fs/2 and fs − 24 kHz folds back
above 24 kHz, where the decimator removes it.

- **polyBLEP** corrects every jump of VA, SYNC, SUPER and RING, including
  VA's moving pulse edge (the width changes every sample and the edge is
  found where it crosses the phase) and SYNC's resets.
- PD's waves only have slope breaks; the FM engines are sums of sines (the
  feedback path is averaged over two samples so it can't break into
  noise). WT-FM's mipmapped tables pick their bandwidth for the modulated
  phase.
- A test runs every engine at bright settings through an FFT and requires
  the inharmonic energy (aliases) in 20 Hz – 20 kHz to stay below −45 dB.

## The rest of the channel

```
 PITCH ─┬─ SWEEP ─┬─ CHAOS ─┬─ JITTER ─► pitch ─┬── +TUNE2 ─┬── +TUNE3 ─┬── +TUNE4
        ▼                                     OSC 1      OSC 2      OSC 3      OSC 4  (the engine, ×4, each × LEVEL × its envelope)
 TRIGGER ─► amp envelopes ─────────────────────┴──────────┴──────────┴──────────┴─► Σ + CLICK + NOISE
        └─► filter envelope ─┐                                                       │
                             ▼                                                       │
 [SAT pre] ◄─────────────────┼───────────────────────────────────────────────────────┘
     ▼                       ▼
 LADDER (cutoff + KEY + F.ENV) ─► EQ ─► [SAT post] ─► OUTPUT ─► CHORUS ─► ECHO ─┬─────────► PAN ─► mix
                                                                                └─► PLATE ─┘
```

- **ENVELOPE**: ATTACK (0.5–200 ms) rises to the hit's velocity; then each
  oscillator decays towards **SUSTAIN** at its own rate (DECAY, 30 ms – 6
  s, times its multiple). After **GATE** (30 ms – 3 s, or HOLD at the top)
  everything fades out with **RELEASE** (3 ms – 3 s). With the gate held and
  no sustain it's a plain percussive decay; a short gate, some sustain and
  a fast release give the flat, chopped stab.
- **SWEEP · TRANSIENT**: a pitch sweep (TIME, bipolar DEPTH, ±4 octaves),
  and the CLICK and NOISE burst. Both run through the filter.
- **CHAOS**: a slow Lorenz attractor used as an LFO that never repeats. RATE,
  DEPTH and a DEST (pitch, X, Y, Z or cutoff), plus JITTER: per-hit random
  offsets of pitch, macros and cutoff.
- **LADDER FILTER**: a 4-pole transistor ladder in zero-delay-feedback form
  (LP12, LP24, and Xpander-style BP and HP mixes). The cutoff follows:
  - **ENV**: the filter envelope, ±8 octaves, further on harder hits. It
    shares the attack and decays to nothing in **F.DEC** (10 ms – 4 s), so
    the stab opens bright and closes before it stops sounding.
  - **KEY**: key tracking. At 100% the cutoff moves with PITCH, so a chord
    keeps its brightness wherever it's played (relative to 100 Hz).
- **SATURATOR**: DRIVE, MIX and four curves (TAPE, TUBE, HARD, FOLD), before
  the filter or after the EQ.
- **EQ**: LO CUT (12 dB/oct, 10–400 Hz), a low shelf, a mid bell and a
  high shelf, ±18 dB. RBJ biquads; a flat band is bypassed.
- **CHORUS**: three taps swept 120° apart by one LFO, wide and hollow. Below
  its X-OVER the signal stays dry and mono.
- **ECHO**: a dub-style ping-pong delay. TIME (20 ms – 1.2 s, unsynced: it
  glides like tape when moved), FDBK (up to 92%) and TONE (a lowpass in the
  loop, 400 Hz – 12 kHz; a fixed highpass keeps the repeats from piling up
  lows, a soft clipper keeps high feedback bounded). AMOUNT is a send, so
  a tail keeps repeating when the next hit of a mutating patch turns it
  down.
- **REVERB**: Xenokussion's per-channel Dattorro plate, fed by the chorus
  and the echo. AMOUNT (post fader), DECAY, TONE and PRE. PAN acts as a
  balance on the stereo signal.

## How close it gets

The reference set is 27 stabs (not part of this repository; put WAVs in
`./examples` to use the tools). The survey found:

- **Register**: the lowest partials sit at 35–165 Hz, but the weight is in
  the low mids: the spectral centroid is 150–530 Hz (median 260 Hz), and
  almost nothing is above 2 kHz.
- **Length**: −20 dB within 140–600 ms of the peak; several hold flat and
  then cut off (a gate).
- **The filter envelope**: the spectrum darkens by up to 4.7× within
  about 100 ms.
- **Chords**: many bodies are stacked intervals a little off equal
  temperament, e.g. 64/77/96 Hz (1 : 1.20 : 1.50, a minor triad).
- **Inharmonic FM**: partials at *c ± n·m* with ratios like 0.86.
- **Space**: mostly wide (side/mid 0.3–1.1), some with echoes.

### Fitting the references

`examples/fit.rs` searches the panel for each reference. It seeds a few
thousand patches across all engines with OSC 1 on one of the reference's
strongest partials (or an octave or a twelfth below) and OSC 2–4 often on
others, refines the best with a sparse (1+1) evolution strategy, and scores
each candidate on a two-resolution log-frequency spectrogram (mean absolute
dB difference, over the reference's length) plus the feature distance
below.

The results, for the presets as shipped (level-matched, rounded):

| Reference | Preset | Engine | Error (dB) | Nearest other reference (dB) | Feature distance |
| --- | --- | --- | --- | --- | --- |
| sample-001 | Feedback Drone | FB-FM | 4.6 | 8.8 | 0.20 |
| sample-002 | Table Swell | WT-FM | 4.4 | 6.9 | 0.42 |
| sample-003 | Ring Cluster | RING | 4.6 | 8.1 | 0.20 |
| sample-004 | Sync Minor Pluck | SYNC | 4.6 | 9.0 | 0.26 |
| sample-005 | Ring Sustain | RING | 4.8 | 9.4 | 0.72 |
| sample-006 | Super Low Chord | SUPER | 4.3 | 9.1 | 0.15 |
| sample-007 | PD Organ Stab | PD | 5.3 | 9.2 | 0.47 |
| sample-008 | Saw Chord Pluck | VA | 3.9 | 6.7 | 0.15 |
| sample-009 | Super Fifths | SUPER | 4.5 | 8.8 | 0.23 |
| sample-010 | PD Pluck | PD | 4.0 | 10.5 | 0.62 |
| sample-011 | Detuned Table | WT-FM | 3.8 | 12.1 | 0.54 |
| sample-012 | Table Ninths | WT-FM | 4.9 | 6.4 | 0.10 |
| sample-013 | Dub Ring Echo | RING | 4.1 | 8.3 | 0.38 |
| sample-014 | Wide Table Gate | WT-FM | 4.7 | 9.9 | 0.52 |
| sample-015 | Short Ring Stab | RING | 5.9 | 11.3 | 0.36 |
| sample-016 | Chorus Saw Stab | VA | 5.3 | 10.0 | 0.44 |
| sample-017 | Dub Echo Chord | WT-FM | 5.2 | 9.3 | 0.70 |
| sample-018 | Feedback Bark | FB-FM | 4.5 | 11.8 | 0.42 |
| sample-019 | Closing Table | WT-FM | 4.7 | 8.2 | 0.26 |
| sample-020 | Mono Super Gate | SUPER | 4.2 | 11.5 | 0.37 |
| sample-021 | Pulse Gate | VA | 4.3 | 9.1 | 0.60 |
| sample-022 | Super Sub Stab | SUPER | 5.9 | 10.8 | 0.35 |
| sample-023 | Metal FM Hold | FB-FM | 4.5 | 5.9 | 0.76 |
| sample-024 | Metal FM Gate | FM | 5.1 | 4.6 | 0.51 |
| sample-025 | Low Table Clang | WT-FM | 4.8 | 11.2 | 0.34 |
| sample-026 | Ring Pluck | RING | 5.5 | 10.8 | 0.57 |
| sample-027 | FM Bell Stab | FM | 5.3 | 11.3 | 0.37 |

- **Mean error 4.7 dB**, against 9.2 dB from each reference to the nearest
  *other* reference. As a yardstick, the same measure rates a reference
  against itself played 25 cents sharp at 2.0 dB and a semitone sharp at
  4.9 dB: a fitted preset is about as close to its reference as the
  reference transposed by a semitone (`stabs yardstick`).
- Every preset is closer to its reference than any other reference is,
  except *Metal FM Gate*: its reference (024) has a near-twin (023) that is
  closer still.
- Every preset's features are within 0.76 of its reference's: closer than
  the median reference is to its nearest neighbour, bar four (0.62–0.76).
- What the fits miss: the echo repeats of 016 come out weaker, 022's
  bright first 50 ms is a little dull, and the references that are cut off
  at the end of the file ring on in the fits (only the reference's length
  is scored).

Spectrogram sheets of reference vs fit: `python3 tools/fit_sheet.py fits
out.png` after `make fit`.

### RND

`random_patch` draws every control from a range taken from the survey, the
fitted patches and a sensitivity analysis of the misses:

- a root of 85–165 Hz;
- OSC 2–4 on 70% of draws a chord tone (fifths, minor thirds and
  sevenths, octaves, fourths, ninths, a few major thirds) up to 25 cents
  off true, on 15% the octave a few cents off, on 15% anywhere from an
  octave down to an octave and a half up; their decays ×0.28–×1 of the
  root's;
- decays of 0.55–1.8 s, gated half the time (0.22–0.7 s, with up to 50%
  sustain);
- always a lowpass at 160–450 Hz for a 100 Hz root (with 20–70% key
  tracking), opened 2–4.8 octaves by a filter envelope that closes in
  0.27–1.2 s; resonance up to 45%;
- a mid EQ of −1 to +4 dB (cutting the low mids takes the stab out of the
  stab), chorus always, echo half the time, always some plate;
- each engine's macros from the part of their range that stays tonal:
  FM, FB-FM and RING ratios of 1–4.6 (below 1 the sidebands land under the
  root), PD that doesn't relax all the way to a sine, and so on.

Then a few rules, which only move free controls:

- OSC 2–4 fold by octaves into 55–330 Hz, and an oscillator within 0.6
  semitones of another moves to the octave furthest from the rest (two
  near-unison sine-like waves beat the stab away a few hundred
  milliseconds in);
- the cutoff moves by a per-engine amount (a synced or unison saw is
  brighter than a sine-based FM at the same cutoff) and stays at 1.3× the
  root or more (below that, the stab is just the filter envelope's burst);
- a low cut at 0.5–0.85 of the root removes FM and ring-modulation
  difference tones;
- the gate stays open 220 ms past the attack, and sustain is only used on
  gated notes.

**Is a random hit a stab like the references?** `analysis::Features`
measures nine things about a hit: the spectral centroid, the frequencies
below which 10/50/90% of the energy lies, the time to the peak, the time
from the peak to −20 dB, how much darker the 100–200 ms after the peak
are than the first 40 ms, the stereo width and the spectral flatness.
Each is scaled by its spread across the references. A hit counts as a match
when it is at least as close to some reference as the loneliest reference
is to its nearest neighbour. Over 6,000 RND hits (MUTATE at RND does
exactly the same per hit) rendered through the whole engine:

- **97.9%** match (seeds of 600–1,000 hits: 97.1–98.7%).
- The median hit is 0.66 from its nearest reference; the references
  themselves are a median 0.60 from their nearest neighbours.
- The misses are mostly long, bright, organ-like holds and a few very
  bright, very short plucks.
- `make stabs` prints the comparison, per engine and per feature, and
  `stabs sensitivity DIR N` shows which draws the misses come from (that
  is how the rules above were found). A unit test holds RND to at least
  95% against the reference features, which are compiled into the DSP crate
  (`analysis::REFERENCES`), so it runs without the WAVs.

## Layout

```
crates/dsp        engine, the 8 engines, wavetables, transient, chaos modulator, ladder,
                  saturator, EQ, chorus, echo, plate reverb, parameters, presets, the hit
                  preview, RND/MUTATE/locks, and the analysis (features, spectrogram
                  distance, the reference table); no dependencies
  src/engines/      va, sync, unison (SUPER), fm, fbfm, ring, pd, wtfm; tables (the wavetable bank)
  examples/         render (presets → WAV), bench, calibrate (loudness survey),
                    analyze (RMS/pitch/brightness over time), probe (which knob made it quiet?),
                    stabs (survey the references; measure RND against them; render RND hits),
                    fit (fit a patch to each reference)
crates/worklet    C ABI over the engine, compiled to a standalone wasm module with no imports
crates/app        Dioxus UI with `web` and `desktop` features; panel state and persistence
tools/            Python (numpy/scipy/matplotlib) helpers: spectrogram sheets of a folder,
                  of fits, and a table of fitted parameters
```

The UI never touches the audio engine directly. It sends small commands (set
a parameter, trigger, master volume): over the worklet's `MessagePort` on the
web, and over an `mpsc` channel on the desktop. Both backends render in
blocks of 128 frames. Locks live only in the UI; the engine never sees them.

## Inspiration and sources

- Basic Channel / Maurizio and the dub techno chord: minor triads and
  sevenths through a lowpass, a tape echo and a spring or plate
- Drumcode-style FM stabs: a sine modulator two octaves up, velocity on the
  FM amount and the cutoff
- Juno-style pulse-width modulation; Roland and Oberheim-style unison
- Yamaha DX operator feedback; Casio **CZ** phase distortion
- V. Välimäki et al., polyBLEP oscillators; R. Bristow-Johnson, *Audio EQ
  Cookbook*; V. Zavalishin, *The Art of VA Filter Design* (ZDF ladder)
- J. Dattorro, "Effect Design, Part 1: Reverberator and Other Filters", JAES 45(9), 1997 (plate reverb)
- E. N. Lorenz, "Deterministic Nonperiodic Flow", J. Atmos. Sci. 20, 1963 (the chaos modulator)
