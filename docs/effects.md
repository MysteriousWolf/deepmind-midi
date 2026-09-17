# Effects

Generated from [`spec/`](../spec) by `cargo xtask docs`. Prose here is written by
hand; everything between the generated markers comes from the specification.

The DeepMind has four FX engines. Each holds twelve raw parameter bytes, at
offsets 167-178, 180-191, 193-204 and 206-217, and what those bytes mean depends
on which of the 35 algorithms the engine is running. `FX 1 Param 3` is Size on a
room reverb and Acceleration on a rotary speaker. This document is the table that
makes those 48 bytes readable, one algorithm at a time.

For the protocol around them (how to address a parameter, how the four engines
are wired to each other, what `FX Type` and `FX Routing` hold) see
[the MIDI specification](midi-spec.md).

## Contents

- [Ranges](#ranges)
- [Which parameters respond to modulation](#which-parameters-respond-to-modulation)
- [Reading a panel](#reading-a-panel)
- [What a slot does](#what-a-slot-does)
- [Glyphs](#glyphs)
- [Marks](#marks)
- [Characters](#characters)
- [Algorithms](#algorithms)
- [Corrections to the manual](#corrections-to-the-manual)

## Ranges

Ranges are what the synthesizer displays, not raw values. Every parameter is
still one byte on the wire; the manual gives no mapping between the two, so only
the two ends are recorded here and the curve between them is unknown. See
[Scaling](midi-spec.md#scaling-raw-values-to-displayed-values) for what can and
cannot be said about that curve.

## Which parameters respond to modulation

All 48 slots are addressable from the modulation matrix as `Fx 1 Param 1`
through `Fx 4 Param 12`, whatever algorithm is loaded, so a blank in the `Mod`
column does not mean unreachable. It means the manual does not mark the
parameter as responding. The pattern is consistent: parameters that select
between options rather than sweep a range are almost all blank, as are
structural ones such as reverb size and pre-delay.

## Reading a panel

Each algorithm is drawn from two figures the manual prints side by side in
section 9.3: the synthesizer's own FX page, which gives the positions, and the
effect's editor panel, which gives the control and the colours.

The positions are a grid, measured from all 35 screenshots, which agree on it:
six controls to a row, in slot order, wrapping onto a second row, left aligned.
The controls are what the panel beside each screenshot uses: 29 effects are
knobs, five are vertical faders on a cream surface, and the Vintage Room Reverb
is numeric displays. The colours are sampled from the same picture. All of it
lives in [`spec/layout.toml`](../spec/layout.toml), which records how each was
taken.

An editor drawing its own panels wants that data and not these drawings, so the
crate publishes it: `effect::grid()` is the grid, `FxSlot::position` is where a
slot lands on it, and `Algorithm::panel` is the control shape and the four
colours. The grid is in the pixels of the 128x64 display it was measured on,
with the display's dimensions beside it, so what a host scales is a proportion
rather than a size.

Each drawing is filed and linked under the `FX Type` value and the manual's full
name, so `31-moog-type-filter` is the Mood Filter. The number is needed: the
manual gives types 32 and 33 the same full name, Dual Pitch Shifter, and only
their short names, `DualPitch` and `Vintage Pitch`, tell them apart.

The drawings say what is on the page, not what a patch sounds like:

| Mark | Meaning |
|---|---|
| Arc behind a knob, ladder beside a fader | The control's full travel |
| No marks | Continuous: the whole travel is in range |
| Two marks | A switch, one mark per state |
| Several marks | A selector, one mark per option |
| Dot above right | The engine acts on modulation reaching this slot |
| Text above | The short name, exactly as the synthesizer's display shows it |
| Text below | The same parameter written out, expanded by this project |

Every handle sits at the middle of its travel, which carries no information.
The specification holds no default value for an effect parameter, so there is
no position to draw, and the middle is the same for every slot on every page.
Leaving the handle off was tried first and read badly: a fader without a cap is
a line with ticks beside it.

## What a slot does

Every slot has a `Control` column below saying what to draw for it. It also has
a quantity, through `FxSlot::quantity`, saying what it does to a signal, and the
two are different questions:

| | |
|---|---|
| `kind` | what control to draw: a knob, a button, a list |
| `quantity` | what picture the slot belongs in |

A `Mix`, a `Feedback` and a `Pre-Delay` are all a byte 0-255 and they do three
unrelated things to a drawing. Nine quantities — time, frequency, gain,
feedback, depth, position, shape, switch, selection — let a host group an
algorithm's slots without matching on titles across 35 algorithms. A third
answer, `FxSlot::glyph`, is finer than either and is for a screen rather than
for grouping: a picture of what the slot does, the size of a character, drawn
beside its value. The [glyphs](#glyphs) are their own section below.

Where the two disagree is the point. A delay's `Factor` is drawn as a selector,
because it picks from ten fractions the manual prints, and it is a *time*: what
it sets is when the tap lands.

Like the titles and the groupings, the quantity is derived from the parameter
rather than transcribed, so it is a convention this project chose.
[`spec/panels.toml`](../spec/panels.toml) records how: the unit decides it
wherever the manual gives one, the name decides otherwise, and where the name
reads the other way from the manual's own description the description wins. A
reverb's `Attack` is a shape, not a time, because the manual calls it the
contour of the reverberation envelope.

## Glyphs

A display with a row of characters per parameter has room for a name and a
number. A controller with a small screen beside each knob has room for less. A
glyph is what goes in that room: a picture of what turning the control does,
the size of a character, on the same seven by seven grid the marks and the
modulation matrix cells are drawn on, so a host blits all three with one
routine.

Every effect slot carries one, through `FxSlot::glyph`, because what a slot
does is the one thing the algorithm always knows about it. A program parameter
carries one through `ParamId::glyph` where one fits, which is every parameter
but the 48 effect slots, whose picture depends on the algorithm, and the
program name's characters, which are letters and not a control. A controller
carries one through `Controller::glyph`: its parameter's where it drives one,
and for the standard controllers what the MIDI specification says they are.
The foot controller is a treadle, expression is the same treadle carrying a
level, and the sustain pedal is a switch under a foot, so a host drawing
pedals draws three different pedals. A modulation matrix cell may name a glyph
rather than draw its own dots, which is the last column of the catalogue: the
mod wheel is one picture whether it is met as a controller or as a source, and
`All Attack` is the attack its three envelopes' parameters carry.

The same glyph serves every control that does the same thing: every `Low Cut`
is a low cut, whether it is on a reverb, a delay or the voice's own filter, and
a host that has learned one has learned them all. Which glyph a parameter
carries is this project's reading of what it does, made once in
[`spec/glyphs.toml`](../spec/glyphs.toml) the way the quantity was, and never
from what the control is called.

<!-- generated:glyphs -->

<img src="diagrams/glyphs.svg" alt="The parameter glyphs, magnified and at one dot per dot" width="596">

| Glyph | Picture of | Effect slots | Parameters | Controllers | Matrix cells |
|---|---|---|---|---|---|
| <a id="glyph-level"></a>`level` | How loud: a wedge rising to the right, the way a fader's travel is printed. | 25 | 6 | 1 | 0 |
| <a id="glyph-mix"></a>`mix` | Wet against dry: a disc half filled. | 31 | 0 | 0 | 0 |
| <a id="glyph-time"></a>`time` | When: a clock face with one hand. | 20 | 5 | 0 | 0 |
| <a id="glyph-pre-delay"></a>`pre delay` | A gap before the effect starts: the dry hit, empty space, then the tail arriving. | 14 | 0 | 0 | 0 |
| <a id="glyph-decay"></a>`decay` | A tail falling away: an exponential fall from full. | 14 | 3 | 0 | 1 |
| <a id="glyph-attack"></a>`attack` | A rise to full and held there: the onset of an envelope. | 7 | 5 | 0 | 1 |
| <a id="glyph-release"></a>`release` | Held, then let go: a level falling to the floor at the end. | 4 | 3 | 0 | 1 |
| <a id="glyph-hold"></a>`hold` | A level held: a line kept up between the moment it starts and the moment it ends. | 5 | 4 | 0 | 1 |
| <a id="glyph-size"></a>`size` | How big: the corners of a box, with what is inside it marked. | 12 | 0 | 0 | 0 |
| <a id="glyph-high-cut"></a>`high cut` | The highs taken off: level, then falling to the right. A low-pass corner. | 27 | 2 | 0 | 0 |
| <a id="glyph-low-cut"></a>`low cut` | The lows taken off: rising from the left, then level. A high-pass corner. | 17 | 1 | 0 | 0 |
| <a id="glyph-low-shelf"></a>`low shelf` | The lows lifted: a shelf above the level at the left. | 10 | 1 | 0 | 0 |
| <a id="glyph-high-shelf"></a>`high shelf` | The highs lifted: a shelf above the level at the right. | 8 | 0 | 0 | 0 |
| <a id="glyph-bell"></a>`bell` | A band lifted about a centre: a bump on a level. | 3 | 0 | 0 | 0 |
| <a id="glyph-resonance"></a>`resonance` | A peak: the response standing up at one frequency. | 6 | 1 | 0 | 0 |
| <a id="glyph-frequency"></a>`frequency` | Where in the spectrum: a ruler whose ticks close up towards the right, the way octaves do. | 8 | 0 | 0 | 0 |
| <a id="glyph-feedback"></a>`feedback` | Sent round again: a loop with an arrowhead on it. | 15 | 0 | 0 | 0 |
| <a id="glyph-rate"></a>`rate` | How fast: a wave, a cycle and a half of it. | 17 | 3 | 0 | 0 |
| <a id="glyph-wave"></a>`wave` | The wave itself: one cycle of a sine. | 5 | 3 | 0 | 0 |
| <a id="glyph-depth"></a>`depth` | How far the wave swings: a cycle between the two rails it reaches. | 15 | 11 | 0 | 0 |
| <a id="glyph-width"></a>`width` | How wide: a span with a mark at each end. | 0 | 0 | 0 | 0 |
| <a id="glyph-spread"></a>`spread` | Pushed apart: arrows pointing both ways from the middle. | 11 | 1 | 0 | 0 |
| <a id="glyph-pan"></a>`pan` | Where between left and right: a dot between the two ways it can go. | 9 | 0 | 2 | 1 |
| <a id="glyph-threshold"></a>`threshold` | A level things happen above: a line across, with one peak over it. | 3 | 0 | 0 | 0 |
| <a id="glyph-ratio"></a>`ratio` | How hard a level is held down past the threshold: a slope with a knee in it. | 3 | 0 | 0 | 0 |
| <a id="glyph-drive"></a>`drive` | Pushed hard: a bolt. | 9 | 0 | 0 | 0 |
| <a id="glyph-tone"></a>`tone` | Tilted: more of one end of the spectrum and less of the other, about a pivot. | 4 | 0 | 0 | 0 |
| <a id="glyph-pitch"></a>`pitch` | A pitch: a note. | 4 | 5 | 0 | 2 |
| <a id="glyph-detune"></a>`detune` | Two pitches almost the same: two waves almost in step. | 5 | 1 | 0 | 3 |
| <a id="glyph-phase"></a>`phase` | Where in the cycle: a wave with its axis through it. | 6 | 0 | 0 | 0 |
| <a id="glyph-selection"></a>`selection` | One of a list: the list. | 14 | 40 | 0 | 0 |
| <a id="glyph-switch"></a>`switch` | In or out: the power ring. | 12 | 3 | 0 | 0 |
| <a id="glyph-diffusion"></a>`diffusion` | Scattered: dots with no line through them. | 10 | 0 | 0 | 0 |
| <a id="glyph-noise"></a>`noise` | Random: dots everywhere. | 0 | 3 | 0 | 0 |
| <a id="glyph-steps"></a>`steps` | In steps: a staircase. | 3 | 33 | 0 | 0 |
| <a id="glyph-tap"></a>`tap` | Repeats: the same mark three times, shorter each time. | 10 | 0 | 0 | 0 |
| <a id="glyph-square"></a>`square` | A pulse: a wave with two levels and nothing between them. | 0 | 2 | 0 | 0 |
| <a id="glyph-saw"></a>`saw` | A saw: a ramp up and a drop. | 0 | 1 | 0 | 0 |
| <a id="glyph-gate"></a>`gate` | How long a note is held open: one pulse and its width. | 0 | 1 | 0 | 0 |
| <a id="glyph-swing"></a>`swing` | Long then short: two pulses that do not share a width. | 0 | 2 | 0 | 0 |
| <a id="glyph-curve"></a>`curve` | Bent away from straight: a line that bows. | 4 | 15 | 0 | 0 |
| <a id="glyph-glide"></a>`glide` | Slid from one pitch to the next rather than stepped. | 0 | 2 | 0 | 0 |
| <a id="glyph-polarity"></a>`polarity` | Which way up: plus over minus. | 0 | 1 | 0 | 0 |
| <a id="glyph-keys"></a>`keys` | The keyboard: keys, with the black ones between them. | 0 | 6 | 0 | 0 |
| <a id="glyph-envelope"></a>`envelope` | An envelope: up fast, down to a level, held, and let go. | 0 | 2 | 0 | 0 |
| <a id="glyph-velocity"></a>`velocity` | How hard the key was hit: an arrow with motion beside it. | 0 | 2 | 0 | 1 |
| <a id="glyph-pressure"></a>`pressure` | Pressed after the key is down: an arrow pressing on a key. | 0 | 3 | 0 | 1 |
| <a id="glyph-bend"></a>`bend` | The pitch bender: a wheel seen from the side, resting in the middle. | 0 | 3 | 0 | 1 |
| <a id="glyph-mod-wheel"></a>`mod wheel` | The modulation wheel: a wheel seen from the side, resting at the bottom. | 0 | 3 | 1 | 1 |
| <a id="glyph-pedal"></a>`pedal` | A pedal under a foot: a treadle over its base, tipped by the toe. | 0 | 0 | 1 | 1 |
| <a id="glyph-expression"></a>`expression` | An expression pedal: the same treadle, carrying a level. | 0 | 0 | 1 | 1 |
| <a id="glyph-footswitch"></a>`footswitch` | A footswitch: a box on the floor with a button on top. | 0 | 0 | 1 | 0 |
| <a id="glyph-cabinet"></a>`cabinet` | A speaker cabinet: a box with a driver in it. | 1 | 0 | 0 | 0 |
| <a id="glyph-breath"></a>`breath` | Breath down a tube: a flow between two walls. | 0 | 0 | 1 | 1 |

<!-- /generated:glyphs -->

## Marks

A host putting all four engines on one page has room for a word or a symbol per
engine, not both. `Multiband Distortion` beside `Stereo Imaging` beside
`Vintage Room Reverb` is three long words the eye reads one at a time.

So each algorithm carries a mark, through `Algorithm::mark`. Like the grid and
the colours it is data rather than a drawing, and it comes two ways: `strokes`
for any size a host can stroke a line at, and `pixels` for a display too small
to stroke anything.

<img src="diagrams/marks.svg" alt="The nine effect family marks and the variants under them, as strokes and as pixels" width="646">

### The marks are diagrams, not pictures

Each one is a diagram of a single idea rather than of the effect. A reverb is
not a drawing of a decaying tail; it is a source with two wavefronts leaving it.
A compressor is not a transfer curve; it is a level with a ceiling and a floor
around it. What a symbol has to do at twelve pixels is be told apart from the
other eight, and a faithful small drawing loses that fight to a reduced one.

The whole set is built from five things and nothing else: a straight run, a
circular arc, a sine, a filled dot, and a corner between two runs. Nothing is
filled except the dots, and no path closes except the rotary ring. Line weight
is the host's and is the same for every mark, which is what makes them read as
one set rather than nine drawings.

The sine is published as a curve rather than as points on one. A modulation
mark drawn as a polyline is a row of corners, and how many corners is a question
about the size the host is drawing at — which is the host's to answer, for the
same reason `generator` hands back a function to sample rather than a picture.
`Stroke::Wave` gives a centre line, an amplitude and a cycle count; the host
samples it as finely as its pixels deserve.

Two pairs are deliberately each other's transpose, because the things they
describe are: Dynamics is a gap between two horizontal limits, Imaging a span
between two vertical ones. Reverb and Rotary are the only marks with curves,
and the ring being closed is what tells them apart.

### The pixel grids

`Mark::pixels` is the same marks drawn again on a seven by seven grid, one
bit per pixel, for an LCD row beside an effect name or a hardware panel.
`Pixels::is_lit` reads one, `Pixels::row` hands a host a row to blit.

They are drawn by hand rather than rasterised from the strokes. At forty-nine
pixels legibility is a question of which pixels, not of scale, and a one-pixel
stroke put through a rasteriser at this size comes out as a grey smear with the
idea gone. Seven is odd, so every mark has a true centre pixel to hang symmetry
on, and it is the smallest grid that fits the ring with a middle in it.

Both renderings are in the drawing above, the grids magnified and again at one
pixel per pixel, which is the size they are actually for. Every one was checked
there before it was written down, and three were redrawn because they did not
survive it.

### Nine families, and a variant where a symbol can carry the difference

The mark belongs first to the family. Every reverb is a source with two
wavefronts leaving it, and every delay is three falling repeats: that is the
idea, and it is what the four engines are told apart by from across the room.

Under the families are the variants, in the second and third rows of the
drawing above, for the kinds within a family that a symbol can carry. A plate
reverb is the family's wavefronts leaving a plate rather than a point; a hall
is the wavefronts a long way from their source; a gated reverb has a wall the
tail stops at; a reverse reverb is the mark turned round. A four-tap delay has
four repeats, the Tel-Ray's come off a can, the decimator's come out in steps.
The chorus is the wave doubled, the flanger the wave braided with its inverse,
the phaser the notches the sweep leaves. Each adds one element to its family's
mark and never a different idea, which is what keeps the set reading as one
set whichever marks four engines land on.

`Algorithm::mark` is the mark to draw: the variant's where the algorithm has
one, the family's otherwise. `Algorithm::own_mark` says which. Where the
difference between two algorithms is not something a symbol can carry, an
ambient reverb against a TC reverb, a stereo delay against a 3-tap, there is
no variant: a drawing that implied a difference it cannot show would be a
guess about which reverb this is, and the name tells those apart.

The family is worth having on its own, through `Algorithm::family`, and is the
half with no pixels in it. `category` is the manual's four buckets, chosen for a
table of contents, and `Creative` holds the phaser, the pitch shifter and the
rotary speaker, which do three unrelated things. A family is the same kind of
published fact at the resolution a symbol needs, and it lets a host group the
delays together whatever page the manual prints them on. Both are published:
the two answer different questions.

Nothing here is measured off the manual, unlike the panel colours. The marks are
drawn by this project, and they are a reading of what each family does rather
than a reproduction of anything the manufacturer prints. No font, no raster and
no licensed symbol set. Where the mark goes is the host's layout, the same way
the grid is data and the pixel size is not.
[`spec/marks.toml`](../spec/marks.toml) is the whole of it.

## Characters

A family is one thing per algorithm and says what it does to a signal. A
character is what kind of thing it is, and an algorithm has any number of
them: the Tel-Ray delay is a delay by family and is vintage, modelled on a
named unit, lo-fi and modulated by character. `Algorithm::characters` reads
them, and a host with four engines on one page has four more ways to tell the
engines apart than the family gives it — a vintage unit drawn as one, a stereo
pair drawn as a pair, two effects in one engine drawn as two.

Every membership carries its reason, and the reasons are of three kinds only:
the manual's own name for the effect, the slots the manual gives it, and the
unit a name refers to where that unit is a matter of record. Nothing is a
character because of how it sounds, and an algorithm listed under none has
nothing to say beyond its family, which is the honest answer rather than a
character invented to fill the row.
[`spec/characters.toml`](../spec/characters.toml) is the whole of it.

<!-- generated:characters -->

#### Vintage

Named vintage, or named for a unit from before effects were digital.

| Algorithm | Because |
|---|---|
| [VintageRev](#03-vintage-room-reverb) | the manual calls it the Vintage Room Reverb |
| [Vintage Pitch](#33-dual-pitch-shifter) | the manual calls it Vintage Pitch |
| [FairComp](#15-compressor) | named for the Fairchild 670, a valve compressor of 1959 |
| [T-RayDelay](#24-tel-ray-delay) | named for the Tel-Ray oil-can delay of the 1960s |
| [Chorus-D](#28-dimensional-chorus) | named for the Roland Dimension D of 1979 |
| [MoodFilter](#31-moog-type-filter) | named for the Moog ladder filter of 1965 |
| [RotarySpkr](#34-rotary-speaker) | a rotary speaker is a mechanical device of the 1940s, and its motor is one of the slots |

#### Modelled

Named for a specific product or maker, so its panel is a picture of that unit.

| Algorithm | Because |
|---|---|
| [TC-DeepVRB](#00-tc-deep-reverb) | TC Electronic's reverb, under their name |
| [MidasEQ](#13-midas-equaliser) | named for the Midas console equaliser |
| [FairComp](#15-compressor) | named for the Fairchild 670 |
| [T-RayDelay](#24-tel-ray-delay) | named for the Tel-Ray oil-can delay |
| [Chorus-D](#28-dimensional-chorus) | named for the Roland Dimension D |
| [EdisonEX1](#18-stereo-imaging) | named for the Behringer Edison EX1 stereo image processor |
| [MoodFilter](#31-moog-type-filter) | the manual calls it the Moog-Type Filter |

#### Stereo

Two channels, with a control per side or a job that is the stereo field itself.

| Algorithm | Because |
|---|---|
| [Delay](#21-stereo-delay) | the manual calls it the Stereo Delay, with a delay factor and a feedback per side |
| [Chorus](#27-stereo-chorus) | the manual calls it the Stereo Chorus, with a width and a delay per side |
| [Flanger](#29-stereo-flanger) | the manual calls it the Stereo Flanger, with a width and a delay per side |
| [Phaser](#30-stereo-phaser) | the manual calls it the Stereo Phaser |
| [EdisonEX1](#18-stereo-imaging) | the manual calls it Stereo Imaging |
| [Auto Pan](#19-auto-panning) | what it moves is stereo position |
| [FairComp](#15-compressor) | a left or mid side and a right or side side, each with its own slots |

#### Dual

Two of the same processor in one engine, each with its own slots.

| Algorithm | Because |
|---|---|
| [DualPitch](#32-dual-pitch-shifter) | the manual calls it the Dual Pitch Shifter, with semitones, cents, delay, gain and pan per channel |
| [Vintage Pitch](#33-dual-pitch-shifter) | the manual calls it the Dual Pitch Shifter, with semitones, cents, delay, feedback and pan per channel |
| [FairComp](#15-compressor) | two channels, each with its own gain, threshold, time and bias |

#### Multiband

Splits the signal into bands and treats each on its own.

| Algorithm | Because |
|---|---|
| [MulBndDist](#16-multiband-distortion) | the manual calls it the Multiband Distortion, with a level and a drive per band and two crossovers |
| [MidasEQ](#13-midas-equaliser) | four bands, each with its own gain and frequency |
| [Enhancer](#14-enhancing-eq) | three bands, each with its own gain |

#### Combined

Two effects in one engine, one after the other.

| Algorithm | Because |
|---|---|
| [ChorusVerb](#10-chorus-and-reverb) | the manual calls it Chorus and Reverb |
| [DelayVerb](#11-delay-and-reverb) | the manual calls it Delay and Reverb |
| [FlangVerb](#12-flanger-and-reverb) | the manual calls it Flanger and Reverb |
| [ModDlyRev](#26-modulation-delay-and-reverb) | the manual calls it Modulation, Delay and Reverb |

#### LoFi

Degrades the signal on purpose.

| Algorithm | Because |
|---|---|
| [DecimDelay](#25-decimator-delay) | the manual calls it the Decimator Delay, with a downsample and a bit depth among its slots |
| [T-RayDelay](#24-tel-ray-delay) | an oil-can delay is dark and unsteady by construction, and Wobble is one of its five slots |

#### Modulated

Something inside it moves on its own: an LFO, a spin or a motor.

| Algorithm | Because |
|---|---|
| [Chorus](#27-stereo-chorus) | a speed, a wave and a phase among its slots |
| [Chorus-D](#28-dimensional-chorus) | a chorus by family |
| [Flanger](#29-stereo-flanger) | a speed and a phase among its slots |
| [Phaser](#30-stereo-phaser) | a speed, a wave and a phase among its slots |
| [MoodFilter](#31-moog-type-filter) | a speed and a wave among its slots |
| [Auto Pan](#19-auto-panning) | a speed, a wave and a phase among its slots |
| [RotarySpkr](#34-rotary-speaker) | two speeds, an acceleration and a motor among its slots |
| [AmbVerb](#01-ambient-reverb) | a modulation depth among its slots |
| [RoomRev](#02-room-reverb) | a spin among its slots |
| [ChamberRev](#05-chamber-reverb) | a spin among its slots |
| [RichPltRev](#07-rich-plate-reverb) | a spin among its slots |
| [HallRev](#04-hall-reverb) | a modulation speed among its slots |
| [PlateRev](#06-plate-reverb) | a modulation speed and depth among its slots |
| [ChorusVerb](#10-chorus-and-reverb) | a speed, a wave and a phase among its slots |
| [FlangVerb](#12-flanger-and-reverb) | a speed and a phase among its slots |
| [ModDlyRev](#26-modulation-delay-and-reverb) | a speed and a depth among its slots |

#### Dynamic

Listens to the level of what goes in and moves with it.

| Algorithm | Because |
|---|---|
| [FairComp](#15-compressor) | a compressor, with a threshold per channel |
| [NoiseGate](#20-noise-gate) | a gate, with a threshold, an attack, a hold and a release |
| [Auto Pan](#19-auto-panning) | an envelope speed and depth among its slots |
| [Phaser](#30-stereo-phaser) | an envelope modulation, an attack, a hold and a release among its slots |
| [MoodFilter](#31-moog-type-filter) | an envelope modulation, an attack and a release among its slots |

<!-- /generated:characters -->

<!-- generated:grid -->

| Property | Value |
|---|---|
| Columns | 6 |
| Rows | 2 |
| Fill order | slot |
| Alignment of a row that is not full | left |
| Shape the FX page draws for every slot | circle |
| Display measured on | 128 x 64 pixels |
| First control centre | 13.5, 14.4 |
| Column pitch | 20.0 |
| Row pitch | 18.6 |
| Control diameter | 11.9 |

The drawings below space their rows further apart than 18.6 pixels, because the synthesizer has room for a three-letter label and these have room for the parameter's name. Everything across a row is as measured. The shape at each position is not: the FX page draws every slot as a circle, and the drawings use what the effect's own panel uses instead. Every handle is drawn at the middle of travel, which carries no meaning.

<!-- /generated:grid -->

<!-- generated:effect-index -->

### Reverb

| `FX Type` | Effect | Name | Slots | Page | Panel |
|---|---|---|---|---|---|
| 0 | [TC-DeepVRB](#00-tc-deep-reverb) | TC Deep Reverb | 5 | one row of 5 | knob |
| 1 | [AmbVerb](#01-ambient-reverb) | Ambient Reverb | 10 | rows of 6 and 4 | fader |
| 2 | [RoomRev](#02-room-reverb) | Room Reverb | 12 | rows of 6 and 6 | fader |
| 3 | [VintageRev](#03-vintage-room-reverb) | Vintage Room Reverb | 12 | rows of 6 and 6 | display |
| 4 | [HallRev](#04-hall-reverb) | Hall Reverb | 12 | rows of 6 and 6 | fader |
| 5 | [ChamberRev](#05-chamber-reverb) | Chamber Reverb | 12 | rows of 6 and 6 | fader |
| 6 | [PlateRev](#06-plate-reverb) | Plate Reverb | 12 | rows of 6 and 6 | knob |
| 7 | [RichPltRev](#07-rich-plate-reverb) | Rich Plate Reverb | 12 | rows of 6 and 6 | fader |
| 8 | [GatedRev](#08-gated-reverb) | Gated Reverb | 10 | rows of 6 and 4 | knob |
| 9 | [Reverse](#09-reverse-reverb) | Reverse Reverb | 9 | rows of 6 and 3 | knob |
| 10 | [ChorusVerb](#10-chorus-and-reverb) | Chorus and Reverb | 12 | rows of 6 and 6 | knob |
| 11 | [DelayVerb](#11-delay-and-reverb) | Delay and Reverb | 12 | rows of 6 and 6 | knob |
| 12 | [FlangVerb](#12-flanger-and-reverb) | Flanger and Reverb | 12 | rows of 6 and 6 | knob |

### Processing

| `FX Type` | Effect | Name | Slots | Page | Panel |
|---|---|---|---|---|---|
| 13 | [MidasEQ](#13-midas-equaliser) | Midas Equaliser | 11 | rows of 6 and 5 | knob |
| 14 | [Enhancer](#14-enhancing-eq) | Enhancing EQ | 9 | rows of 6 and 3 | knob |
| 15 | [FairComp](#15-compressor) | Compressor | 12 | rows of 6 and 6 | knob |
| 16 | [MulBndDist](#16-multiband-distortion) | Multiband Distortion | 12 | rows of 6 and 6 | knob |
| 17 | [RackAmp](#17-rack-amplifier) | Rack Amplifier | 9 | rows of 6 and 3 | knob |
| 18 | [EdisonEX1](#18-stereo-imaging) | Stereo Imaging | 8 | rows of 6 and 2 | knob |
| 19 | [Auto Pan](#19-auto-panning) | Auto Panning | 9 | rows of 6 and 3 | knob |
| 20 | [NoiseGate](#20-noise-gate) | Noise Gate | 8 | rows of 6 and 2 | knob |

### Delay

| `FX Type` | Effect | Name | Slots | Page | Panel |
|---|---|---|---|---|---|
| 21 | [Delay](#21-stereo-delay) | Stereo Delay | 12 | rows of 6 and 6 | knob |
| 22 | [3TapDelay](#22-3-tap-delay) | 3-Tap Delay | 12 | rows of 6 and 6 | knob |
| 23 | [4TapDelay](#23-4-tap-delay) | 4-Tap Delay | 12 | rows of 6 and 6 | knob |
| 24 | [T-RayDelay](#24-tel-ray-delay) | Tel-Ray Delay | 5 | one row of 5 | knob |
| 25 | [DecimDelay](#25-decimator-delay) | Decimator Delay | 12 | rows of 6 and 6 | knob |
| 26 | [ModDlyRev](#26-modulation-delay-and-reverb) | Modulation, Delay and Reverb | 12 | rows of 6 and 6 | knob |

### Creative

| `FX Type` | Effect | Name | Slots | Page | Panel |
|---|---|---|---|---|---|
| 27 | [Chorus](#27-stereo-chorus) | Stereo Chorus | 11 | rows of 6 and 5 | knob |
| 28 | [Chorus-D](#28-dimensional-chorus) | Dimensional Chorus | 7 | rows of 6 and 1 | knob |
| 29 | [Flanger](#29-stereo-flanger) | Stereo Flanger | 12 | rows of 6 and 6 | knob |
| 30 | [Phaser](#30-stereo-phaser) | Stereo Phaser | 12 | rows of 6 and 6 | knob |
| 31 | [MoodFilter](#31-moog-type-filter) | Moog-Type Filter | 12 | rows of 6 and 6 | knob |
| 32 | [DualPitch](#32-dual-pitch-shifter) | Dual Pitch Shifter | 12 | rows of 6 and 6 | knob |
| 33 | [Vintage Pitch](#33-dual-pitch-shifter) | Dual Pitch Shifter | 12 | rows of 6 and 6 | knob |
| 34 | [RotarySpkr](#34-rotary-speaker) | Rotary Speaker | 8 | rows of 6 and 2 | knob |

<!-- /generated:effect-index -->

## Algorithms

<!-- generated:effects -->

<a id="00-tc-deep-reverb"></a>

### TC Deep Reverb (TC-DeepVRB)

<img src="diagrams/fx/00-tc-deep-reverb.svg" alt="TC Deep Reverb front panel" width="720">

`FX Type` 0, reverb, drawn with the reverb mark. 5 slots, drawn as knobs. Characters: [Modelled](#modelled).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PST` | Preset | Preset | selector |  | Ambience, Church, Gate, Hall, Lo Fi, Modulated, Plate, Room, Spring, Tile, Default |  | [selection](#glyph-selection) |  |
| 2 | `DCY` | Decay | Decay | continuous |  | 0.1 to 6.0 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate (range is preset dependant). |
| 3 | `TON` | Tone | Tone | continuous |  | -50.0 to 50.0 % | yes | [tone](#glyph-tone) | Enhances high frequencies/low frequencies for positive/negative settings respectivly. |
| 4 | `PDY` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 5 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |

<a id="01-ambient-reverb"></a>

### Ambient Reverb (AmbVerb)

<img src="diagrams/fx/01-ambient-reverb.svg" alt="Ambient Reverb front panel" width="720">

`FX Type` 1, reverb, drawn with the reverb mark. 10 slots, drawn as faders. Characters: [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PD` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 0.2 to 7.3 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `SIZ` | Size | Size | continuous |  | 2.0 to 100.0 |  | [size](#glyph-size) | Controls the perceived size of the space being created by the reverb. |
| 4 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the decay of the high frequencies within the reverb tail. |
| 5 | `DIF` | Diffusion | Diffusion | continuous |  | 1.0 to 30.0 |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the reverb to be reduced. |
| 9 | `MOD` | Mod | Modulation Depth | continuous |  | 0.0 to 100 % |  | [depth](#glyph-depth) | Controls the reverb tail modulation depth. |
| 10 | `TGN` | TailGain | Tail Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Adjusts the volume of the reverb tail. |

<a id="02-room-reverb"></a>

### Room Reverb (RoomRev)

<img src="diagrams/fx/02-room-reverb.svg" alt="Room Reverb front panel" width="720">

`FX Type` 2, reverb, drawn with the room mark of the reverb family. 12 slots, drawn as faders. Characters: [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PRE` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 0.3 to 28.9 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `SIZ` | Size | Size | continuous |  | 4.0 to 76.0 m |  | [size](#glyph-size) | Controls the perceived size of the space being created by the reverb. |
| 4 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the decay of the high frequencies within the reverb tail. |
| 5 | `DIF` | Diffusion | Diffusion | continuous |  | 0.0 to 100 % |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the reverb to be reduced. |
| 9 | `LFX` | BassMult | Bass Multiplier | continuous |  | 0.2 to 4.0 | yes | [low shelf](#glyph-low-shelf) | Controls the low frequency build-up. |
| 10 | `SPR` | Spread | Spread | continuous |  | 0.0 to 50.0 |  | [spread](#glyph-spread) | Emphasizes the stereo effect of the reverb. |
| 11 | `SHP` | Shape | Shape | continuous |  | 0.0 to 250.0 | yes | [curve](#glyph-curve) | Adjusts the contour of the reverberation envelope. |
| 12 | `SPI` | Spin | Spin | continuous |  | 0.0 to 100 % |  | [rate](#glyph-rate) | Controls randomization / modulation effects within the reverb. |

<a id="03-vintage-room-reverb"></a>

### Vintage Room Reverb (VintageRev)

<img src="diagrams/fx/03-vintage-room-reverb.svg" alt="Vintage Room Reverb front panel" width="720">

`FX Type` 3, reverb, drawn with the room mark of the reverb family. 12 slots, drawn as displays. Characters: [Vintage](#vintage).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PRE` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `SIZ` | Size | Size | continuous |  | 1.0 to 100 % |  | [size](#glyph-size) | Controls the perceived size of the space being created by the reverb. Affects DECAY range. |
| 3 | `DCY` | Decay | Decay | continuous |  | 0.1 to 20.7 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. Range dependant on SIZE. |
| 4 | `LFX` | Lo Mult | Low Multiplier | continuous |  | 0.1 to 10.0 | yes | [low shelf](#glyph-low-shelf) | Controls the low frequency build-up. |
| 5 | `HFX` | Hi Mult | High Multiplier | continuous |  | 0.1 to 10.0 | yes | [high shelf](#glyph-high-shelf) | Controls the high frequency build-up. |
| 6 | `DEN` | Density | Density | continuous |  | 0.0 to 100 % | yes | [diffusion](#glyph-diffusion) | Manipulates the reflection density in the simulated room. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the reverb to be reduced. |
| 9 | `ERL` | ER Level | Early Reflection Level | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Set the early reflection times. |
| 10 | `ERD` | ER Delay | Early Reflection Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Sets the loudness of the early reflection level. |
| 11 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 12 | `FRZ` | Freeze | Freeze | switch |  | OFF to ON | yes | [hold](#glyph-hold) | Applies freeze mode and blends signals into a continuous response. |

<a id="04-hall-reverb"></a>

### Hall Reverb (HallRev)

<img src="diagrams/fx/04-hall-reverb.svg" alt="Hall Reverb front panel" width="720">

`FX Type` 4, reverb, drawn with the hall mark of the reverb family. 12 slots, drawn as faders. Characters: [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PD` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 0.2 to 4.9 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `SIZ` | Size | Size | continuous |  | 2.0 to 200.0 |  | [size](#glyph-size) | Controls the perceived size of the space being created by the reverb. |
| 4 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the decay of the high frequencies within the reverb tail. |
| 5 | `DIF` | Diffusion | Diffusion | continuous |  | 1.0 to 30.0 |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the reverb to be reduced. |
| 9 | `LFX` | BassMult | Bass Multiplier | continuous |  | 0.5 to 2.0 | yes | [low shelf](#glyph-low-shelf) | Controls the low frequency build-up. |
| 10 | `SPR` | Spread | Spread | continuous |  | 0.0 to 50.0 |  | [spread](#glyph-spread) | Emphasizes the stereo effect of the reverb. |
| 11 | `SHP` | Shape | Shape | continuous |  | 0.0 to 250.0 | yes | [curve](#glyph-curve) | Adjusts the contour of the reverberation envelope. |
| 12 | `MOD` | ModSpeed | Modulation Speed | continuous |  | 0.0 to 100.0 |  | [rate](#glyph-rate) | Controls the reverb tail modulation rate . |

<a id="05-chamber-reverb"></a>

### Chamber Reverb (ChamberRev)

<img src="diagrams/fx/05-chamber-reverb.svg" alt="Chamber Reverb front panel" width="720">

`FX Type` 5, reverb, drawn with the room mark of the reverb family. 12 slots, drawn as faders. Characters: [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PRE` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 0.3 to 28.9 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `SIZ` | Size | Size | continuous |  | 4.0 to 76.0 m |  | [size](#glyph-size) | Controls the perceived size of the space being created by the reverb. |
| 4 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the decay of the high frequencies within the reverb tail. |
| 5 | `DIF` | Diffusion | Diffusion | continuous |  | 0.0 to 100 % |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the reverb to be reduced. |
| 9 | `LFX` | BassMult | Bass Multiplier | continuous |  | 0.2 to 4.0 | yes | [low shelf](#glyph-low-shelf) | Controls the low frequency build-up. |
| 10 | `SPR` | Spread | Spread | continuous |  | 0.0 to 50.0 |  | [spread](#glyph-spread) | Emphasizes the stereo effect of the reverb. |
| 11 | `SHP` | Shape | Shape | continuous |  | 0.0 to 250.0 | yes | [curve](#glyph-curve) | Adjusts the contour of the reverberation envelope. |
| 12 | `SPI` | Spin | Spin | continuous |  | 0.0 to 100 % |  | [rate](#glyph-rate) | Controls randomization / modulation effects within the reverb. |

<a id="06-plate-reverb"></a>

### Plate Reverb (PlateRev)

<img src="diagrams/fx/06-plate-reverb.svg" alt="Plate Reverb front panel" width="720">

`FX Type` 6, reverb, drawn with the plate mark of the reverb family. 12 slots, drawn as knobs. Characters: [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PD` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 0.5 to 10.0 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `SIZ` | Size | Size | continuous |  | 2.0 to 200.0 |  | [size](#glyph-size) | Controls the perceived size of the space being created by the reverb. |
| 4 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the decay of the high frequencies within the reverb tail. |
| 5 | `DIF` | Diffusion | Diffusion | continuous |  | 1.0 to 30.0 |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the reverb to be reduced. |
| 9 | `LFX` | BassMult | Bass Multiplier | continuous |  | 0.5 to 2.0 | yes | [low shelf](#glyph-low-shelf) | Controls the low frequency build-up. |
| 10 | `XOV` | Xover | Crossover | continuous |  | 10.0 to 500.0 Hz | yes | [frequency](#glyph-frequency) | Controls the crossover point for bass multiplier. |
| 11 | `MOD` | ModDepth | Modulation Depth | continuous |  | 1.0 to 50.0 |  | [depth](#glyph-depth) | Controls the reverb tail modulation depth. |
| 12 | `MDS` | ModSpeed | Modulation Speed | continuous |  | 0.0 to 100.0 |  | [rate](#glyph-rate) | Controls the reverb tail modulation rate. |

<a id="07-rich-plate-reverb"></a>

### Rich Plate Reverb (RichPltRev)

<img src="diagrams/fx/07-rich-plate-reverb.svg" alt="Rich Plate Reverb front panel" width="720">

`FX Type` 7, reverb, drawn with the plate mark of the reverb family. 12 slots, drawn as faders. Characters: [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PD` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 0.3 to 28.9 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `SIZ` | Size | Size | continuous |  | 4.0 to 39.0 m |  | [size](#glyph-size) | Controls the perceived size of the space being created by the reverb. |
| 4 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the decay of the high frequencies within the reverb tail. |
| 5 | `DIF` | Diffusion | Diffusion | continuous |  | 0.0 to 100 % |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the reverb to be reduced. |
| 9 | `LFX` | BassMult | Bass Multiplier | continuous |  | 0.2 to 4.0 | yes | [low shelf](#glyph-low-shelf) | Controls the low frequency build-up. |
| 10 | `SPR` | Spread | Spread | continuous |  | 0.0 to 50.0 |  | [spread](#glyph-spread) | Emphasizes the stereo effect of the reverb. |
| 11 | `ATK` | Attack | Attack | continuous |  | 0.0 to 100.0 | yes | [attack](#glyph-attack) | Adjusts the contour of the reverberation envelope. |
| 12 | `SPN` | Spin | Spin | continuous |  | 0.0 to 100 % | yes | [rate](#glyph-rate) | Controls randomization / modulation effects within the reverb. |

<a id="08-gated-reverb"></a>

### Gated Reverb (GatedRev)

<img src="diagrams/fx/08-gated-reverb.svg" alt="Gated Reverb front panel" width="720">

`FX Type` 8, reverb, drawn with the gated mark of the reverb family. 10 slots, drawn as knobs.

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PD` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 140.0 to 1000.0 ms |  | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `ATK` | Attack | Attack | continuous |  | 0.0 to 30.0 | yes | [attack](#glyph-attack) | Adjusts the contour of the reverberation envelope . |
| 4 | `DEN` | Density | Density | continuous |  | 1.0 to 50.0 | yes | [diffusion](#glyph-diffusion) | Manipulates the reflection density in the simulated room. |
| 5 | `SPR` | Spread | Spread | continuous |  | 0.0 to 100.0 |  | [spread](#glyph-spread) | Emphasizes the stereo effect of the reverb. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HIF` | HiSvFreq | High Shelf Frequency | continuous |  | 200.0 to 20000.0 Hz | yes | [high shelf](#glyph-high-shelf) | Adjusts the frequency of a Hi-Shelving filter at the input of the reverb effect. |
| 9 | `HIG` | HiSvGain | High Shelf Gain | continuous |  | -30.0 to 0.0 dB | yes | [high shelf](#glyph-high-shelf) | Adjusts the gain of a Hi-Shelving filter at the input of the reverb effect. |
| 10 | `DIF` | Diffusion | Diffusion | continuous |  | 0.0 to 100 % |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density . |

<a id="09-reverse-reverb"></a>

### Reverse Reverb (Reverse)

<img src="diagrams/fx/09-reverse-reverb.svg" alt="Reverse Reverb front panel" width="720">

`FX Type` 9, reverb, drawn with the reverse mark of the reverb family. 9 slots, drawn as knobs.

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PD` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 2 | `DCY` | Decay | Decay | continuous |  | 140.0 to 1000.0 ms |  | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 3 | `RIS` | Rise | Rise | continuous |  | 0.0 to 50.0 | yes | [attack](#glyph-attack) | Controls how quickly the effect builds up. |
| 4 | `DIF` | Diffusion | Diffusion | continuous |  | 1.0 to 30.0 |  | [diffusion](#glyph-diffusion) | Controls the initial reflection density. |
| 5 | `SPR` | Spread | Spread | continuous |  | 0.0 to 100.0 |  | [spread](#glyph-spread) | Controls how the reflection is distributed through the envelope of the reverb. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies affected by the reverb to be reduced. |
| 8 | `HIF` | HiSvFreq | High Shelf Frequency | continuous |  | 200.0 to 20000.0 Hz | yes | [high shelf](#glyph-high-shelf) | Adjusts the frequency of a Hi-Shelving filter at the input of the reverb effect. |
| 9 | `HIG` | HiSvGain | High Shelf Gain | continuous |  | -30.0 to 0.0 dB | yes | [high shelf](#glyph-high-shelf) | Adjusts the gain of a Hi-Shelving filter at the input of the reverb effect. |

<a id="10-chorus-and-reverb"></a>

### Chorus and Reverb (ChorusVerb)

<img src="diagrams/fx/10-chorus-and-reverb.svg" alt="Chorus and Reverb front panel" width="720">

`FX Type` 10, reverb, drawn with the chorused mark of the reverb family. 12 slots, drawn as knobs. Characters: [Combined](#combined), [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SPD` | Speed | Speed | continuous |  | 0.0 to 4.0 Hz | yes | [rate](#glyph-rate) | Adjusts the rate of the chorus. Time synchronised options from 4 to 1/64 bars. |
| 2 | `DEP` | Depth | Depth | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Adjusts the modulation depth of the chorus. |
| 3 | `DLY` | Delay | Delay | continuous |  | 0.5 to 50.0 ms |  | [time](#glyph-time) | Adjusts the delay of the chorus. |
| 4 | `PHS` | Phase | Phase | continuous |  | 0.0 to 180.0 | yes | [phase](#glyph-phase) | Offsets the phase between the left and right channels. |
| 5 | `WAV` | Wave | Wave | continuous |  | 0.0 to 100 % |  | [wave](#glyph-wave) | Adjusts the LFO waveform from a sine wave to triangular wave. |
| 6 | `BAL` | Balance | Balance | continuous |  | -100.0 to 100.0 | yes | [mix](#glyph-mix) | Adjusts the balance between chorus and reverb. |
| 7 | `PRE` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 8 | `DCY` | Decay | Decay | continuous |  | 0.1 to 5.0 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 9 | `SIZ` | Size | Size | continuous |  | 2.0 to 200.0 |  | [size](#glyph-size) | Controls how large or small the simulated space is. |
| 10 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Determines the decay of high frequencies within the reverb tail. |
| 11 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Excludes low frequencies below the value . |
| 12 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |

<a id="11-delay-and-reverb"></a>

### Delay and Reverb (DelayVerb)

<img src="diagrams/fx/11-delay-and-reverb.svg" alt="Delay and Reverb front panel" width="720">

`FX Type` 11, reverb, drawn with the repeated mark of the reverb family. 12 slots, drawn as knobs. Characters: [Combined](#combined).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `TIM` | Time | Time | continuous |  | 1.0 to 1500.0 ms |  | [time](#glyph-time) | Adjusts the delay time for the left channel delay. Time sync options from 4 to 1/64 bars. |
| 2 | `PAT` | Pattern | Delay Pattern | selector |  | 1/4, 1X |  | [selection](#glyph-selection) | Sets the delay ratio for the right channel delay. |
| 3 | `FHC` | FeedHC | Feedback High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Trims high frequencies from the feedback. |
| 4 | `FBK` | Feedback | Feedback | continuous |  | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Controls the percentage of feedback. |
| 5 | `XFD` | X-Feed | Cross-Feedback | continuous |  | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Control the amount of delay sound sent to the reverb effect. |
| 6 | `BAL` | Balance | Balance | continuous |  | -100.0 to 100.0 | yes | [mix](#glyph-mix) | Adjusts the ratio between delay and reverb. |
| 7 | `PRE` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 8 | `DCY` | Decay | Decay | continuous |  | 0.1 to 5.0 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 9 | `SIZ` | Size | Size | continuous |  | 2.0 to 200.0 |  | [size](#glyph-size) | Controls how large or small the simulated space is. |
| 10 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Determines the decay of high frequencies within the reverb tail. |
| 11 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Excludes low frequencies below the value . |
| 12 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |

<a id="12-flanger-and-reverb"></a>

### Flanger and Reverb (FlangVerb)

<img src="diagrams/fx/12-flanger-and-reverb.svg" alt="Flanger and Reverb front panel" width="720">

`FX Type` 12, reverb, drawn with the chorused mark of the reverb family. 12 slots, drawn as knobs. Characters: [Combined](#combined), [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SPD` | Speed | Speed | continuous |  | 0.0 to 4.0 Hz | yes | [rate](#glyph-rate) | Adjusts the rate of the flanger. Time synchronised options from 4 to 1/64 bars. |
| 2 | `DEP` | Depth | Depth | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Adjusts the modulation depth of the flanger. |
| 3 | `DLY` | Delay | Delay | continuous |  | 0.5 to 20.0 ms |  | [time](#glyph-time) | Adjusts the delay of the flanger. |
| 4 | `PHS` | Phase | Phase | continuous |  | 0.0 to 180.0 | yes | [phase](#glyph-phase) | Offsets the phase between the left and right channels. |
| 5 | `FBK` | Feed | Feedback | continuous |  | -90.0 to 90.0 % | yes | [feedback](#glyph-feedback) | Controls the percentage of positive or negative feedback. |
| 6 | `BAL` | Balance | Balance | continuous |  | -100.0 to 100.0 | yes | [mix](#glyph-mix) | Adjusts the balance between flanger and reverb. |
| 7 | `PRE` | PreDelay | Pre-Delay | continuous |  | 0.0 to 200.0 ms |  | [pre delay](#glyph-pre-delay) | Controls the amount of time before the reverb is heard following the source signal. |
| 8 | `DCY` | Decay | Decay | continuous |  | 0.1 to 5.0 s | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 9 | `SIZ` | Size | Size | continuous |  | 2.0 to 200.0 |  | [size](#glyph-size) | Controls how large or small the simulated space is. |
| 10 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Determines the decay of high frequencies within the reverb tail. |
| 11 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Excludes low frequencies below the value . |
| 12 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |

<a id="13-midas-equaliser"></a>

### Midas Equaliser (MidasEQ)

<img src="diagrams/fx/13-midas-equaliser.svg" alt="Midas Equaliser front panel" width="720">

`FX Type` 13, processing, drawn with the equaliser mark of the filter family. 11 slots, drawn as knobs. Characters: [Modelled](#modelled), [Multiband](#multiband).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `LSG` | LoShelfGain | Low Shelf Gain | continuous | low | -12.0 to 12.0 dB | yes | [low shelf](#glyph-low-shelf) | Adjusts the gain of the low band. |
| 2 | `LSF` | LoShelfFreq | Low Shelf Frequency | continuous | low | 30.0 to 20000.0 Hz | yes | [low shelf](#glyph-low-shelf) | Adjusts the frequency of the low band. |
| 3 | `LMG` | LoMidGain | Low-Mid Gain | continuous | low-mid | -12.0 to 12.0 dB | yes | [bell](#glyph-bell) | Adjusts the gain of the low-mid band. |
| 4 | `LMF` | LoMidFreq | Low-Mid Frequency | continuous | low-mid | 30.0 to 20000.0 Hz | yes | [frequency](#glyph-frequency) | Adjusts the frequency of the low-mid band. |
| 5 | `LMQ` | LoMidQ | Low-Mid Q | continuous | low-mid | 0.3 to 5.0 | yes | [resonance](#glyph-resonance) | Adjusts the Q-factor of the low-mid band. |
| 6 | `HMG` | HiMidGain | High-Mid Gain | continuous | high-mid | -12.0 to 12.0 dB | yes | [bell](#glyph-bell) | Adjusts the gain of the high-mid band. |
| 7 | `HMF` | HiMidFreq | High-Mid Frequency | continuous | high-mid | 30.0 to 20000.0 Hz | yes | [frequency](#glyph-frequency) | Adjusts the frequency of the high- mid band. |
| 8 | `HMQ` | HiMidQ | High-Mid Q | continuous | high-mid | 0.3 to 5.0 | yes | [resonance](#glyph-resonance) | Adjusts the Q-factor of the high-mid band. |
| 9 | `HSG` | HiShelfGain | High Shelf Gain | continuous | high | -12.0 to 12.0 dB | yes | [high shelf](#glyph-high-shelf) | Adjusts the gain of the high band. |
| 10 | `HSF` | HiShelfFreq | High Shelf Frequency | continuous | high | 30.0 to 20000.0 Hz | yes | [high shelf](#glyph-high-shelf) | Adjusts the frequency of the high band. |
| 11 | `EQ` | EQ | EQ In or Out | switch |  | IN, OUT | yes | [switch](#glyph-switch) | INOUTAdjusts the frequency of the high band. |

<a id="14-enhancing-eq"></a>

### Enhancing EQ (Enhancer)

<img src="diagrams/fx/14-enhancing-eq.svg" alt="Enhancing EQ front panel" width="720">

`FX Type` 14, processing, drawn with the equaliser mark of the filter family. 9 slots, drawn as knobs. Characters: [Multiband](#multiband).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `OGN` | OutGain | Output Gain | continuous |  | -12.0 to 12.0 dB | yes | [level](#glyph-level) | Compensates for changes in level resulting from the effect. |
| 2 | `SPR` | Spread | Spread | continuous |  | 0.0 to 100 % | yes | [spread](#glyph-spread) | Emphasizes the stereo content for a wider mix . |
| 3 | `BGN` | BassGain | Bass Gain | continuous | low | 0.0 to 100 % | yes | [low shelf](#glyph-low-shelf) | Adjusts the gain of the bass band. |
| 4 | `BFR` | BassFreq | Bass Frequency | continuous | low | 1.0 to 50.0 | yes | [low shelf](#glyph-low-shelf) | Adjusts the frequency of the bass band. |
| 5 | `MGN` | MidGain | Mid Gain | continuous | mid | 0.0 to 100 % | yes | [bell](#glyph-bell) | Adjusts the gain of the mid band. |
| 6 | `MIQ` | MidQ | Mid Q | continuous | mid | 1.0 to 50.0 | yes | [resonance](#glyph-resonance) | Adjusts the Q-factor of the mid band. |
| 7 | `HIG` | HiGain | High Gain | continuous | high | 0.0 to 100 % | yes | [high shelf](#glyph-high-shelf) | Adjusts the gain of the high band. |
| 8 | `HIF` | HiFreq | High Frequency | continuous | high | 1.0 to 50.0 | yes | [frequency](#glyph-frequency) | Adjusts the frequency of the high band. |
| 9 | `SOL` | Solo | Solo | switch |  | OFF, ON |  | [switch](#glyph-switch) | Solo mode - used to isolate only the audio resulting from the effect. |

<a id="15-compressor"></a>

### Compressor (FairComp)

<img src="diagrams/fx/15-compressor.svg" alt="Compressor front panel" width="720">

`FX Type` 15, processing, drawn with the dynamics mark. 12 slots, drawn as knobs. Characters: [Vintage](#vintage), [Modelled](#modelled), [Stereo](#stereo), [Dual](#dual), [Dynamic](#dynamic).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `MOD` | Mode | Mode | selector |  | Off, Stereo, Dual, M/S |  | [selection](#glyph-selection) | Mode of operation: Off, Stereo, Dual, M/S (Mid/Side). |
| 2 | `INL` | InGain L/M | Input Gain, left/mid | continuous | left-mid | -20.0 to 0.0 | yes | [level](#glyph-level) | Controls the input gain for the Left/Mid signal. |
| 3 | `THL` | Thresh L/M | Threshold, left/mid | continuous | left-mid | 0.0 to 10.0 | yes | [threshold](#glyph-threshold) | Controls the threshold for the Left/Mid signal. |
| 4 | `TML` | Time L/M | Time, left/mid | continuous | left-mid | 1.0 to 6.0 |  | [time](#glyph-time) | Controls the attack and release time for the Left/Mid channel. |
| 5 | `DCL` | DC Bias L/M | DC Bias, left/mid | continuous | left-mid | 0.0 to 100 % | yes | [ratio](#glyph-ratio) | Adjust the ratio and knee of the compression curve for the Left/Mid signal. |
| 6 | `OGL` | OutGain L/M | Output Gain, left/mid | continuous | left-mid | -18.0 to 6.0 dB | yes | [level](#glyph-level) | Controls the output gain for the Left/Mid signal. |
| 7 | `BAL` | Bias Bal | Bias Balance | continuous |  | -100.0 to 100 % | yes | [ratio](#glyph-ratio) | Adjust the bias current, creating accentuation of attacks |
| 8 | `INR` | InGain R/S | Input Gain, right/side | continuous | right-side | -20.0 to 0.0 | yes | [level](#glyph-level) | Controls the input gain for the Right/Side signal. |
| 9 | `THR` | Thresh R/S | Threshold, right/side | continuous | right-side | 0.0 to 10.0 | yes | [threshold](#glyph-threshold) | Controls the threshold for the Right/Side signal. |
| 10 | `TMR` | Time R/S | Time, right/side | continuous | right-side | 1.0 to 6.0 |  | [time](#glyph-time) | Controls the attack and release time for the Right/Side channel. |
| 11 | `DCR` | DC Bias R/S | DC Bias, right/side | continuous | right-side | 0.0 to 100 % | yes | [ratio](#glyph-ratio) | Adjust the ratio and knee of the compression curve for the Right/Side signal. |
| 12 | `OGR` | OutGain R/S | Output Gain, right/side | continuous | right-side | -18.0 to 6.0 dB | yes | [level](#glyph-level) | Controls the output gain for the Right/Side signal. |

<a id="16-multiband-distortion"></a>

### Multiband Distortion (MulBndDist)

<img src="diagrams/fx/16-multiband-distortion.svg" alt="Multiband Distortion front panel" width="720">

`FX Type` 16, processing, drawn with the banded mark of the distortion family. 12 slots, drawn as knobs. Characters: [Multiband](#multiband).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `IPG` | InputGain | Input Gain | continuous |  | -24.0 to 24.0 dB | yes | [level](#glyph-level) | Controls the amount of input gain applied to the signal. |
| 2 | `DST` | Dist Types | Distortion Type | selector |  | VAL, SAT, TUB, PFV, PFS, PFT |  | [selection](#glyph-selection) | Distortion Type: VAL(Valve), SAT(Saturation), TUB (Tube), & Post Filter variants (PFV/PFS/PFT). |
| 3 | `LBL` | Low Level | Low Band Level | continuous | low | -12.0 to 12.0 dB | yes | [level](#glyph-level) | Controls the level of the frequencies below XoverFreq 1. |
| 4 | `LDR` | Low Drive | Low Band Drive | continuous | low | 0.0 to 100 % | yes | [drive](#glyph-drive) | Controls the percentage of distortion introduced below XoverFreq 1. |
| 5 | `XV1` | Xover Freq 1 | Crossover Frequency 1 | continuous |  | 30.0 to 9000.0 Hz | yes | [frequency](#glyph-frequency) | Sets the lower cross over frequency. |
| 6 | `MBL` | Mid Level | Mid Band Level | continuous | mid | -12.0 to 12.0 dB | yes | [level](#glyph-level) | Controls the level of the frequencies between Xover1 Freq and Xover2 Freq. |
| 7 | `MDR` | Mid Drive | Mid Band Drive | continuous | mid | 0.0 to 100 % | yes | [drive](#glyph-drive) | Controls the percentage of distortion introduced between Xover1 Freq and Xover2 Freq. |
| 8 | `XV2` | Xover Freq 2 | Crossover Frequency 2 | continuous |  | 30.0 to 9000.0 Hz | yes | [frequency](#glyph-frequency) | Sets the upper cross over frequency. |
| 9 | `HBL` | High Level | High Band Level | continuous | high | -12.0 to 12.0 dB | yes | [level](#glyph-level) | Controls the level of the frequencies below XoverFreq 2. |
| 10 | `HDR` | High Drive | High Band Drive | continuous | high | 0.0 to 100 % | yes | [drive](#glyph-drive) | Controls the percentage of distortion introduced above XoverFreq 2. |
| 11 | `CAB` | Cabinet | Cabinet | selector |  | OFF, VTw, VBs, A10, Mid, BFC, B60, V30, S78, Oax, Ac1, Ac2 |  | [selection](#glyph-selection) | Cabinet Type: OFF, VTw, VBs, A10, Mid, BFC, B60, V30, S78, Oax, A12, Rck. (See Table Above). |
| 12 | `OPG` | OutputGain | Output Gain | continuous |  | -12.0 to 12.0 dB | yes | [level](#glyph-level) | Controls the amount of output gain applied to the signal. |

<a id="17-rack-amplifier"></a>

### Rack Amplifier (RackAmp)

<img src="diagrams/fx/17-rack-amplifier.svg" alt="Rack Amplifier front panel" width="720">

`FX Type` 17, processing, drawn with the amplifier mark of the distortion family. 9 slots, drawn as knobs.

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `PRE` | PreAmp | Pre-Amp | continuous |  | 0.0 to 10.0 | yes | [level](#glyph-level) | Adjusts the amount of input gain prior to the band-specific distortion adjustment. |
| 2 | `BUZ` | Buzz | Buzz | continuous |  | 0.0 to 10.0 | yes | [drive](#glyph-drive) | Adjusts the amount of low-end breakup. |
| 3 | `PNC` | Punch | Punch | continuous |  | 0.0 to 10.0 | yes | [drive](#glyph-drive) | Adjusts the amount of midrange distortion. |
| 4 | `CRN` | Crunch | Crunch | continuous |  | 0.0 to 10.0 | yes | [drive](#glyph-drive) | Tailors the high-frequency content and distortion for smooth or cutting notes. |
| 5 | `DRV` | Drive | Drive | continuous |  | 0.0 to 10.0 | yes | [drive](#glyph-drive) | Emulates the amount of power amp distortion from a tube amp. |
| 6 | `LVL` | Level | Level | continuous |  | 0.0 to 10.0 | yes | [level](#glyph-level) | Controls the overall output level. |
| 7 | `LOW` | Low | Low Tone | continuous |  | 0.0 to 10.0 | yes | [tone](#glyph-tone) | EQ adjustment of the low frequencies, independent of distortion content. |
| 8 | `HI` | High | High Tone | continuous |  | 0.0 to 10.0 | yes | [tone](#glyph-tone) | EQ adjustment of the high frequencies, independent of distortion content. |
| 9 | `CAB` | Cabinet | Cabinet | switch |  | OFF to ON |  | [cabinet](#glyph-cabinet) | Turns the cabinet simulation on or off. |

<a id="18-stereo-imaging"></a>

### Stereo Imaging (EdisonEX1)

<img src="diagrams/fx/18-stereo-imaging.svg" alt="Stereo Imaging front panel" width="720">

`FX Type` 18, processing, drawn with the imaging mark. 8 slots, drawn as knobs. Characters: [Modelled](#modelled), [Stereo](#stereo).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `ON` | On | On or Off | switch |  | OFF to ON |  | [switch](#glyph-switch) | Allows the effect to be turned On or Off. |
| 2 | `IMD` | InMode | Input Mode | selector |  | ST, M/S |  | [selection](#glyph-selection) | STM/SControls the input mode (Stereo or Mid/Side). |
| 3 | `OMD` | OutMode | Output Mode | selector |  | ST, M/S |  | [selection](#glyph-selection) | STM/SControls the output mode (Stereo or Mid/Side). |
| 4 | `STS` | StSpread | Stereo Spread | continuous |  | -50.0 to 50.0 | yes | [spread](#glyph-spread) | Controls the spread of the stereo field. |
| 5 | `LMF` | LMF Spread | Low-Mid Frequency Spread | continuous |  | -50.0 to 50.0 | yes | [spread](#glyph-spread) | Controls the spread of the stereo field for low/mid frequencies only. |
| 6 | `BAL` | Balance | Balance | continuous |  | -50.0 to 50.0 | yes | [pan](#glyph-pan) | Adjusts the ratio of mono to stereo content. |
| 7 | `CNT` | CntrDist | Centre Distance | continuous |  | -50.0 to 50.0 | yes | [size](#glyph-size) | Allows the mono content to be panned. |
| 8 | `GN` | Gain | Gain | continuous |  | -12.0 to 12.0 dB | yes | [level](#glyph-level) | Controls the amount of output gain applied to the signal. |

<a id="19-auto-panning"></a>

### Auto Panning (Auto Pan)

<img src="diagrams/fx/19-auto-panning.svg" alt="Auto Panning front panel" width="720">

`FX Type` 19, processing, drawn with the panning mark of the imaging family. 9 slots, drawn as knobs. Characters: [Stereo](#stereo), [Modulated](#modulated), [Dynamic](#dynamic).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SPD` | Speed | Speed | continuous |  | 0.0 to 5.0 Hz | yes | [rate](#glyph-rate) | Adjusts the LFO rate. Time synchronised options from 4 to 1/64 bars. |
| 2 | `PHS` | Phase | Phase | continuous |  | 0.0 to 180.0 | yes | [phase](#glyph-phase) | Controls the LFO phase difference between the left and right channels. |
| 3 | `WAV` | Wave | Wave | continuous |  | -50.0 to 50.0 | yes | [wave](#glyph-wave) | Blends the LFO waveform between triangular and square shape. |
| 4 | `DEP` | Depth | Depth | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Controls the depth (amount) of modulation. |
| 5 | `ESP` | EnvSpd | Envelope Speed | continuous |  | 0.0 to 100 % | yes | [rate](#glyph-rate) | Adjusts how much the LFO speed is modulated by the envelope. |
| 6 | `EDP` | EnvDepth | Envelope Depth | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Adjusts the depth of the envelope modulation. |
| 7 | `ATK` | Attack | Attack | continuous |  | 10.0 to 1000.0 ms | yes | [attack](#glyph-attack) | Controls the envelope attack stage time. |
| 8 | `HLD` | Hold | Hold | continuous |  | 1.0 to 2000.0 ms | yes | [hold](#glyph-hold) | Controls the envelope hold stage time. |
| 9 | `REL` | Release | Release | continuous |  | 10.0 to 1000.0 ms | yes | [release](#glyph-release) | Controls the envelope release stage time. |

<a id="20-noise-gate"></a>

### Noise Gate (NoiseGate)

<img src="diagrams/fx/20-noise-gate.svg" alt="Noise Gate front panel" width="720">

`FX Type` 20, processing, drawn with the gate mark of the dynamics family. 8 slots, drawn as knobs. Characters: [Dynamic](#dynamic).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `THR` | Threshold | Threshold | continuous |  | -50.0 to 0.0 dB | yes | [threshold](#glyph-threshold) | Sets signal level at which gate opens. |
| 2 | `RNG` | Range | Range | continuous |  | -100.0 to 0.0 dB | yes | [level](#glyph-level) | Control adjusts the amount of gain reduction applied to the signal below threshold. |
| 3 | `ATT` | Attack | Attack | continuous |  | 0.0 to 20.0 ms | yes | [attack](#glyph-attack) | Adjusts time taken for gate to open after an over-threshold signal. |
| 4 | `REL` | Release | Release | continuous |  | 2.0 to 1999.9 ms | yes | [release](#glyph-release) | Adjusts time taken for gate to close after programme material falls back below threshold. |
| 5 | `HLD` | Hold | Hold | continuous |  | 2.0 to 1999.9 ms | yes | [hold](#glyph-hold) | This defines a waiting period before the gate starts to close. |
| 6 | `PUN` | Punch | Punch | continuous |  | -6.0 to 6.0 | yes | [drive](#glyph-drive) | Used to increase tonal shaping or reduce gated breathing/delay/resonant howl-round. |
| 7 | `MOD` | Mode | Mode | selector |  | GAT, TRN, DUC |  | [selection](#glyph-selection) | GAT (Gate), TRN (Transient Gate), DUC (Ducker). |
| 8 | `PWR` | Power | Power | switch |  | ON to OFF |  | [switch](#glyph-switch) | Enables gate in the signal path. When switched off, gate is bypassed. |

<a id="21-stereo-delay"></a>

### Stereo Delay (Delay)

<img src="diagrams/fx/21-stereo-delay.svg" alt="Stereo Delay front panel" width="720">

`FX Type` 21, delay, drawn with the delay mark. 12 slots, drawn as knobs. Characters: [Stereo](#stereo).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 2 | `TIM` | Time | Time | continuous |  | 1.0 to 1500.0 ms |  | [time](#glyph-time) | Adjusts the master delay time. Time synchronised options from 4 to 1/64 bars. |
| 3 | `MOD` | Mode | Mode | selector |  | ST, X, M, P-P |  | [selection](#glyph-selection) | ST- Stereo feedback, X - crosses feedback between channels, M - Mono mix in feedback chain. P-P - Ping Pong (Note that Feedback-R (FBR) is disabled in this mode. |
| 4 | `FCL` | FactorL | Delay Factor, left | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Sets left delay to rhythmic fractions of the master delay. |
| 5 | `FCR` | FactorR | Delay Factor, right | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Sets right delay to rhythmic fractions of the master delay. |
| 6 | `OFS` | Offset | Offset | continuous |  | -100.0 to 100.0 ms |  | [time](#glyph-time) | Adds a delay difference between the left and right delayed signals. |
| 7 | `LC` | LC | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Adjusts the low frequency cut, allowing lower frequencies to remain unaffected by the delay. |
| 8 | `HC` | HC | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the high frequency cut, allowing higher frequencies to remain unaffected by the delay. |
| 9 | `FLC` | FeedLC | Feedback Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Adjusts the low cut filter frequency in the feedback paths. |
| 10 | `FBL` | FeedL | Feedback, left | continuous |  | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Control the amount of feedback for the left channel. |
| 11 | `FBR` | FeedR | Feedback, right | continuous |  | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Control the amount of feedback for the right channel. |
| 12 | `FHC` | FeedHC | Feedback High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the high cut filter frequency in the feedback paths. |

<a id="22-3-tap-delay"></a>

### 3-Tap Delay (3TapDelay)

<img src="diagrams/fx/22-3-tap-delay.svg" alt="3-Tap Delay front panel" width="720">

`FX Type` 22, delay, drawn with the delay mark. 12 slots, drawn as knobs.

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `TIM` | Time | Time | continuous |  | 1.0 to 1500.0 ms |  | [time](#glyph-time) | Sets the master delay time, and the first stage. Time synchronised options from 4 to 1/64 bars. |
| 2 | `GNT` | GainT | Tap T Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Sets the gain level of the first stage of the delay. |
| 3 | `PNT` | PanT | Tap T Pan | continuous |  | -100 to 100 % | yes | [pan](#glyph-pan) | Sets the position of the first delay stage in the stereo field. |
| 4 | `FBK` | Feedback | Feedback | continuous |  | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Adjusts the amount of feedback. |
| 5 | `FCA` | FactorA | Tap A Delay Factor | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Controls the delay time in the second stage of the delay. |
| 6 | `GNA` | GainA | Tap A Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Controls the gain level of the second delay stage. |
| 7 | `PNA` | PanA | Tap A Pan | continuous |  | -100 to 100 % | yes | [pan](#glyph-pan) | Sets the position of the second delay stage in the stereo field. |
| 8 | `FCB` | FactorB | Tap B Delay Factor | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Controls the delay time in the third stage of the delay. |
| 9 | `GNB` | GainB | Tap B Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Controls the gain level of the third delay stage. |
| 10 | `PNB` | PanB | Tap B Pan | continuous |  | -100 to 100 % | yes | [pan](#glyph-pan) | Sets the position of the third gain stage in the stereo field. |
| 11 | `XFD` | X-Feed | Cross-Feedback | switch |  | OFF to ON |  | [feedback](#glyph-feedback) | Turns the stereo cross-feedback of the delays On or Off . |
| 12 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |

<a id="23-4-tap-delay"></a>

### 4-Tap Delay (4TapDelay)

<img src="diagrams/fx/23-4-tap-delay.svg" alt="4-Tap Delay front panel" width="720">

`FX Type` 23, delay, drawn with the four taps mark of the delay family. 12 slots, drawn as knobs.

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `TIM` | Time | Time | continuous |  | 1.0 to 1500.0 ms |  | [time](#glyph-time) | Sets the master delay time, and the first stage. Time synchronised options from 4 to 1/64 bars. |
| 2 | `GN` | Gain | Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Sets the gain level of the first stage of the delay. |
| 3 | `FBK` | Feedback | Feedback | continuous |  | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Adjusts the amount of feedback. |
| 4 | `SPR` | Spread | Spread | continuous |  | 0.0 to 6.0 |  | [spread](#glyph-spread) | Positions the first delay stage in the stereo field. |
| 5 | `FCA` | FactorA | Tap A Delay Factor | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Controls the delay time in the second stage of the delay. |
| 6 | `GNA` | GainA | Tap A Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Controls the gain level of the second delay stage. |
| 7 | `FCB` | FactorB | Tap B Delay Factor | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Controls the delay time in the third stage of the delay. |
| 8 | `GNB` | GainB | Tap B Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Controls the gain level of the third delay stage. |
| 9 | `FCC` | FactorC | Tap C Delay Factor | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Controls the delay time in the fourth stage of the delay. |
| 10 | `GNC` | GainC | Tap C Gain | continuous |  | 0.0 to 100 % | yes | [level](#glyph-level) | Controls the gain level of the fourth delay stage . |
| 11 | `XFD` | X-Feed | Cross-Feedback | continuous |  | 0.0 to 1.0 |  | [feedback](#glyph-feedback) | Turns the stereo cross-feedback of the delays On or Off. |
| 12 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |

<a id="24-tel-ray-delay"></a>

### Tel-Ray Delay (T-RayDelay)

<img src="diagrams/fx/24-tel-ray-delay.svg" alt="Tel-Ray Delay front panel" width="720">

`FX Type` 24, delay, drawn with the oil can mark of the delay family. 5 slots, drawn as knobs. Characters: [Vintage](#vintage), [Modelled](#modelled), [LoFi](#lofi).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 2 | `DLY` | Delay | Delay | continuous |  | 0.0 to 100 % | yes | [time](#glyph-time) | Adjusts the master delay time. |
| 3 | `SUS` | Sustain | Sustain | continuous |  | 0.0 to 100 % | yes | [hold](#glyph-hold) | Controls how long the delay is sustained for. Warning at 100% build up will occur. |
| 4 | `WOB` | Wobble | Wobble | continuous |  | 0.0 to 100 % | yes | [detune](#glyph-detune) | Adjusts the amount of wobble caused by age and quality of build/materials. |
| 5 | `TON` | Tone | Tone | continuous |  | 0.0 to 100 % | yes | [tone](#glyph-tone) | Controls the tone of the delays. |

<a id="25-decimator-delay"></a>

### Decimator Delay (DecimDelay)

<img src="diagrams/fx/25-decimator-delay.svg" alt="Decimator Delay front panel" width="720">

`FX Type` 25, delay, drawn with the decimated mark of the delay family. 12 slots, drawn as knobs. Characters: [LoFi](#lofi).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `MIX` | Mix | Mix | continuous |  | 0.00 to 100.00 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 2 | `TIM` | Time M | Master Time | continuous |  | 1.0 to 1500.0 ms |  | [time](#glyph-time) | Adjusts the master delay time. Time synchronized options from 4 to 1/64 bars. |
| 3 | `DSM` | Downsample | Downsample | continuous |  | 0.00 to 100.00 % | yes | [steps](#glyph-steps) | Decimates the signal by reducing the sampling frequency. |
| 4 | `FCL` | FactorL | Delay Factor, left | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Sets left delay to rhythmic fractions of the master delay. |
| 5 | `FCR` | FactorR | Delay Factor, right | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Sets right delay to rhythmic fractions of the master delay. |
| 6 | `BRC` | Bit-Reduce | Bit Depth | continuous |  | 24 to 1. Counts down: 24 bits at the minimum, 1 bit at the maximum. |  | [steps](#glyph-steps) | Decimates the signal by reducing the bit-depth. |
| 7 | `FC` | Cutoff | Cutoff Frequency | continuous |  | 30.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Adjust the cutoff frequency of the filter, allowing specific frequencies to be affected by the delay. |
| 8 | `RES` | Resonance | Resonance | continuous |  | 0.00 to 100.00 % | yes | [resonance](#glyph-resonance) | Adjusts the resonance of the filter. |
| 9 | `FLT` | Type | Type | selector |  | Lowpass, Highpass, Bandpass, Notch |  | [selection](#glyph-selection) |  |
| 10 | `FBL` | FeedL | Feedback, left | continuous |  | 0.00 to 100.00 % | yes | [feedback](#glyph-feedback) | Controls the amount of feedback for the left channel. |
| 11 | `FBR` | FeedR | Feedback, right | continuous |  | 0.00 to 100.00 % | yes | [feedback](#glyph-feedback) | Controls the amount of feedback for the right channel. |
| 12 | `DMT` | Decimate | Decimation Point | switch |  | PRE, POST |  | [switch](#glyph-switch) | It sets the decimation on the input signal (PRE) or only on the delay (POST). |

<a id="26-modulation-delay-and-reverb"></a>

### Modulation, Delay and Reverb (ModDlyRev)

<img src="diagrams/fx/26-modulation-delay-and-reverb.svg" alt="Modulation, Delay and Reverb front panel" width="720">

`FX Type` 26, delay, drawn with the into reverb mark of the delay family. 12 slots, drawn as knobs. Characters: [Combined](#combined), [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `TIM` | Time | Time | continuous |  | 1.0 to 1500.0 ms |  | [time](#glyph-time) | Sets the master delay time. Time synchronised options from 4 to 1/64 bars. |
| 2 | `FAC` | Factor | Delay Factor | selector |  | 1/4, 1/3, 1/2, 2/3, 3/4, 1, 4/3, 3/2, 2, 3 |  | [tap](#glyph-tap) | Sets the delay to rhythmic fractions. |
| 3 | `FBK` | Feedback | Feedback | continuous |  | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Controls the percentage of positive feedback. |
| 4 | `FHC` | FeedHC | Feedback High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the high cut filter frequency in the feedback path. |
| 5 | `DEP` | Depth | Depth | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Controls the depth (amount) of modulation. |
| 6 | `SPD` | Speed | Speed | continuous |  | 0.0 to 10.0 Hz | yes | [rate](#glyph-rate) | Adjusts the rate of the modulation. Time synchronised options from 4 to 1/64 bars. |
| 7 | `MOD` | Mode | Mode | selector |  | PAR, SER |  | [selection](#glyph-selection) | PARSERControls the processor chain routing (Serial/Parallel) |
| 8 | `RTY` | Rtype | Reverb Type | selector |  | AMB, CLUB, HALL |  | [selection](#glyph-selection) | Reverb Type can be set to AMB (Ambience), CLUB, or HALL. |
| 9 | `DCY` | Decay | Decay | continuous |  | 1.0 to 10.0 | yes | [decay](#glyph-decay) | Controls the amount of time it takes for the reverb to dissipate. |
| 10 | `DMP` | Damping | Damping | continuous |  | 1000 to 20000 Hz | yes | [high cut](#glyph-high-cut) | Determines the decay of high frequencies within the reverb tail. |
| 11 | `BAL` | Balance | Balance | continuous |  | -100.0 to 100.0 | yes | [mix](#glyph-mix) | Adjusts ratio of the delay signal to the reverb signal. |
| 12 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals |

<a id="27-stereo-chorus"></a>

### Stereo Chorus (Chorus)

<img src="diagrams/fx/27-stereo-chorus.svg" alt="Stereo Chorus front panel" width="720">

`FX Type` 27, creative, drawn with the doubled mark of the modulation family. 11 slots, drawn as knobs. Characters: [Stereo](#stereo), [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SPD` | Speed | Speed | continuous |  | 0.0 to 5 Hz | yes | [rate](#glyph-rate) | Sets the modulation speed. Time synchronised options from 4 to 1/64 bars. |
| 2 | `WDL` | WidthL | Width, left | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Determines the amount of modulated delay in the left channel. |
| 3 | `WDR` | WidthR | Width, right | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Determines the amount of modulated delay in the right channel. |
| 4 | `DLL` | DelayL | Delay, left | continuous |  | 0.5 to 50.0 ms |  | [time](#glyph-time) | Sets the total amount of delay for the left channel. |
| 5 | `DLR` | DelayR | Delay, right | continuous |  | 0.5 to 50.0 ms |  | [time](#glyph-time) | Sets the total amount of delay for the right channel. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies in the signal to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies in the signal to be reduced. |
| 9 | `PHS` | Phase | Phase | continuous |  | 0.0 to 100.0 |  | [phase](#glyph-phase) | Adjusts the phase offset of the LFO between left and right channel. |
| 10 | `WAV` | Wave | Wave | continuous |  | 0.0 to 100 % |  | [wave](#glyph-wave) | Blends between the digital triangular chorus sound and the classic analog sine wave. |
| 11 | `SPR` | Spread | Spread | continuous |  | 0.0 to 100 % | yes | [spread](#glyph-spread) | Adjusts how much of the left channel is mixed into the right and vice versa. |

<a id="28-dimensional-chorus"></a>

### Dimensional Chorus (Chorus-D)

<img src="diagrams/fx/28-dimensional-chorus.svg" alt="Dimensional Chorus front panel" width="720">

`FX Type` 28, creative, drawn with the doubled mark of the modulation family. 7 slots, drawn as knobs. Characters: [Vintage](#vintage), [Modelled](#modelled), [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `ON` | On | On or Off | switch |  | OFF to ON |  | [switch](#glyph-switch) | Allows the effect to be turned On or Off. |
| 2 | `MOD` | Mode | Mode | selector |  | M, ST |  | [selection](#glyph-selection) | MSTSwitches the operation between Mono and Stereo modes. |
| 3 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 4 | `SW1` | Sw1 | Switch 1 | switch | channel-1 | OFF to ON |  | [switch](#glyph-switch) | Engages level one intensity (minimum). |
| 5 | `SW2` | Sw2 | Switch 2 | switch | channel-2 | OFF to ON |  | [switch](#glyph-switch) | Engages level two intensity. |
| 6 | `SW3` | Sw3 | Switch 3 | switch |  | OFF to ON |  | [switch](#glyph-switch) | Engages level three intensity. |
| 7 | `SW4` | Sw4 | Switch 4 | switch |  | OFF to ON |  | [switch](#glyph-switch) | Engages level four intensity (maximum). |

<a id="29-stereo-flanger"></a>

### Stereo Flanger (Flanger)

<img src="diagrams/fx/29-stereo-flanger.svg" alt="Stereo Flanger front panel" width="720">

`FX Type` 29, creative, drawn with the braided mark of the modulation family. 12 slots, drawn as knobs. Characters: [Stereo](#stereo), [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SPD` | Speed | Speed | continuous |  | 0.0 to 5 Hz | yes | [rate](#glyph-rate) | Sets the modulation speed. Time synchronised options from 4 to 1/64 bars. |
| 2 | `WDL` | WidthL | Width, left | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Determines the amount of modulated delay in the left channel. |
| 3 | `WDR` | WidthR | Width, right | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Determines the amount of modulated delay in the right channel. |
| 4 | `DLL` | DelayL | Delay, left | continuous |  | 0.5 to 20.0 ms |  | [time](#glyph-time) | Sets the total amount of delay for the left channel. |
| 5 | `DLR` | DelayR | Delay, right | continuous |  | 0.5 to 20.0 ms |  | [time](#glyph-time) | Sets the total amount of delay for the right channel. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `LC` | LoCut | Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Allows the low frequencies in the signal to be reduced. |
| 8 | `HC` | Hi Cut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies in the signal to be reduced. |
| 9 | `PHS` | Phase | Phase | continuous |  | 0.0 to 180.0 |  | [phase](#glyph-phase) | Adjusts the phase offset of the LFO between left and right channel. |
| 10 | `FLC` | FeedLC | Feedback Low Cut | continuous |  | 10.0 to 500.0 Hz | yes | [low cut](#glyph-low-cut) | Adjusts the low cut filter frequency in the feedback path. |
| 11 | `FHC` | FeedHC | Feedback High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Adjusts the high cut filter frequency in the feedback path. |
| 12 | `FD` | Feed | Feedback | continuous |  | -90.0 to 90.0 % | yes | [feedback](#glyph-feedback) | Controls the percentage of positive or negative feedback. |

<a id="30-stereo-phaser"></a>

### Stereo Phaser (Phaser)

<img src="diagrams/fx/30-stereo-phaser.svg" alt="Stereo Phaser front panel" width="720">

`FX Type` 30, creative, drawn with the notched mark of the modulation family. 12 slots, drawn as knobs. Characters: [Stereo](#stereo), [Modulated](#modulated), [Dynamic](#dynamic).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SPD` | Speed | Speed | continuous |  | 0.0 to 5 Hz | yes | [rate](#glyph-rate) | Sets the modulation speed. Time synchronised options from 4 to 1/64 bars. |
| 2 | `DEP` | Depth | Depth | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Controls the depth (amount) of modulation. |
| 3 | `RES` | Reso | Resonance | continuous |  | 0.0 to 100 % | yes | [resonance](#glyph-resonance) | Adjusts the resonance of the multiple filter stages. |
| 4 | `BAS` | Base Freq | Base Frequency | continuous |  | 20 to 15000 Hz | yes | [frequency](#glyph-frequency) | Adjusts the frequency range of the modulated filters. |
| 5 | `STG` | Stages | Stages | continuous |  | 2.0 to 12.0 |  | [steps](#glyph-steps) | Controls how many filter stages are used. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `WAV` | Wave | Wave | continuous |  | -50.0 to 50.0 |  | [wave](#glyph-wave) | Shapes the symmetry of the LFO waveform. |
| 8 | `PHS` | Phase | Phase | continuous |  | 0.0 to 180.0 deg |  | [phase](#glyph-phase) | Adjusts the LFO phase difference between the left and right channel |
| 9 | `ENV` | EnvMod | Envelope Modulation | continuous |  | -100.0 to 100 % | yes | [depth](#glyph-depth) | Adjusts the level of positive or negative envelope modulation. |
| 10 | `ATK` | Attack | Attack | continuous |  | 10.0 to 1000.0 ms | yes | [attack](#glyph-attack) | Controls the envelope attack stage time . |
| 11 | `HLD` | Hold | Hold | continuous |  | 1.0 to 2000.0 ms | yes | [hold](#glyph-hold) | Controls the envelope hold stage time. |
| 12 | `REL` | Release | Release | continuous |  | 10.0 to 1000.0 ms | yes | [release](#glyph-release) | Controls the envelope release stage time. |

<a id="31-moog-type-filter"></a>

### Moog-Type Filter (MoodFilter)

<img src="diagrams/fx/31-moog-type-filter.svg" alt="Moog-Type Filter front panel" width="720">

`FX Type` 31, creative, drawn with the resonant mark of the filter family. 12 slots, drawn as knobs. Characters: [Vintage](#vintage), [Modelled](#modelled), [Modulated](#modulated), [Dynamic](#dynamic).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SPD` | Speed | Speed | continuous |  | 0.0 to 20.0 Hz |  | [rate](#glyph-rate) | Sets the modulation speed. Time synchronised options from 4 to 1/64 bars. |
| 2 | `DEP` | Depth | Depth | continuous |  | 0.0 to 100 % | yes | [depth](#glyph-depth) | Controls the depth (amount) of modulation. |
| 3 | `RES` | Resonance | Resonance | continuous |  | 0.0 to 100 % | yes | [resonance](#glyph-resonance) | Adjusts the resonance of the filter. |
| 4 | `FRQ` | Base Freq | Base Frequency | continuous |  | 20 to 15000 Hz | yes | [frequency](#glyph-frequency) | Adjusts the level of positive or negative envelope modulation. |
| 5 | `TYP` | Type | Type | selector |  | LP, HP, BP, NOT |  | [selection](#glyph-selection) | Selects between low pass (LP), high-pass (HP), band- pass (BP) and Notch (NOT). |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `WAV` | Wave | Wave | selector |  | Triangle, Sine, Saw Up, Saw Down, Square, Random, Envelope |  | [wave](#glyph-wave) | Selects modulation waveform Triangular, Sine, Saw Up, Saw Down, Ramp, Square or Random. |
| 8 | `ENV` | EnvMod | Envelope Modulation | continuous |  | -100.0 to 100 % | yes | [depth](#glyph-depth) | Adjusts the level of positive or negative envelope modulation. |
| 9 | `ATK` | Attack | Attack | continuous |  | 10.0 to 249.9 ms | yes | [attack](#glyph-attack) | Controls the filter attack time. |
| 10 | `REL` | Release | Release | continuous |  | 10.0 to 500.0 ms | yes | [release](#glyph-release) | Controls the filter release time. |
| 11 | `DRV` | Drive | Drive | continuous |  | 0.0 to 100 % | yes | [drive](#glyph-drive) | Adjusts the level, also introduces an overdrive effect if pushed hard. |
| 12 | `4P` | 4-Pole | 4-Pole | switch |  | 2P, 4P |  | [switch](#glyph-switch) | Adjusts the slope of the filter from 2-pole to 4-pole (steeper). |

<a id="32-dual-pitch-shifter"></a>

### Dual Pitch Shifter (DualPitch)

<img src="diagrams/fx/32-dual-pitch-shifter.svg" alt="Dual Pitch Shifter front panel" width="720">

`FX Type` 32, creative, drawn with the two intervals mark of the pitch family. 12 slots, drawn as knobs. Characters: [Dual](#dual).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SM1` | Semi1 | Semitones, channel 1 | continuous | channel-1 | -12.0 to 12.0 | yes | [pitch](#glyph-pitch) | Adjusts the pitch of the first channel in semi-tones. |
| 2 | `CN1` | Cent1 | Cents, channel 1 | continuous | channel-1 | -50.0 to 50.0 | yes | [detune](#glyph-detune) | Adjusts the pitch of the first channel in cents. |
| 3 | `DL1` | Delay1 | Delay, channel 1 | continuous | channel-1 | 1.0 to 500.0 ms |  | [time](#glyph-time) | Adjusts the time difference between the wet and dry signals. Sync options from 4 to 1/64 bars. |
| 4 | `GN1` | Gain1 | Gain, channel 1 | continuous | channel-1 | 0.0 to 100 % | yes | [level](#glyph-level) | Allows gain compensation to be applied to the first channel. |
| 5 | `PN1` | Pan1 | Pan, channel 1 | continuous | channel-1 | -100.0 to 100 % | yes | [pan](#glyph-pan) | Allows panning of the first channel. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `SM2` | Semi2 | Semitones, channel 2 | continuous | channel-2 | -12.0 to 12.0 | yes | [pitch](#glyph-pitch) | Adjusts the pitch of the second channel in semi-tones. |
| 8 | `CN2` | Cent2 | Cents, channel 2 | continuous | channel-2 | -50.0 to 50.0 | yes | [detune](#glyph-detune) | Adjusts the pitch of the second channel in cents . |
| 9 | `DL2` | Delay2 | Delay, channel 2 | continuous | channel-2 | 1.0 to 500.0 ms |  | [time](#glyph-time) | Adjusts the time difference between the wet and dry signals. Sync options from 4 to 1/64 bars. |
| 10 | `GN2` | Gain2 | Gain, channel 2 | continuous | channel-2 | 0.0 to 100 % | yes | [level](#glyph-level) | Allows gain compensation to be applied to the second channel. |
| 11 | `PN2` | Pan2 | Pan, channel 2 | continuous | channel-2 | -100.0 to 100 % | yes | [pan](#glyph-pan) | Allows panning of the second channel. |
| 12 | `HIC` | HiCut | High Cut | continuous |  | 200.0 to 20000.0 Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the pitch shifting to be reduced. |

<a id="33-dual-pitch-shifter"></a>

### Dual Pitch Shifter (Vintage Pitch)

<img src="diagrams/fx/33-dual-pitch-shifter.svg" alt="Dual Pitch Shifter front panel" width="720">

`FX Type` 33, creative, drawn with the two intervals mark of the pitch family. 12 slots, drawn as knobs. Characters: [Vintage](#vintage), [Dual](#dual).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `SM1` | Semi1 | Semitones, channel 1 | continuous | channel-1 | -12.0 to 12.0 | yes | [pitch](#glyph-pitch) | Adjusts the pitch of the first channel in semi-tones. |
| 2 | `CN1` | Cent1 | Cents, channel 1 | continuous | channel-1 | -50.0 to 50.0 | yes | [detune](#glyph-detune) | Adjusts the pitch of the first channel in cents. |
| 3 | `DL1` | Delay1 | Delay, channel 1 | continuous | channel-1 | 1.0 to 500.0 ms |  | [time](#glyph-time) | Adjusts the time difference between the wet and dry signals. Sync options from 4 to 1/64 bars. |
| 4 | `FB1` | Feedback1 | Feedback, channel 1 | continuous | channel-1 | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Allows feedback to be applied to the first channel. |
| 5 | `PN1` | Pan1 | Pan, channel 1 | continuous | channel-1 | -100.0 to 100 % | yes | [pan](#glyph-pan) | Allows panning of the first channel. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `SM2` | Semi2 | Semitones, channel 2 | continuous | channel-2 | -12.0 to 12.0 | yes | [pitch](#glyph-pitch) | Adjusts the pitch of the second channel in semi-tones. |
| 8 | `CN2` | Cent2 | Cents, channel 2 | continuous | channel-2 | -50.0 to 50.0 | yes | [detune](#glyph-detune) | Adjusts the pitch of the second channel in cents . |
| 9 | `DL2` | Delay2 | Delay, channel 2 | continuous | channel-2 | 1.0 to 500.0 ms |  | [time](#glyph-time) | Adjusts the time difference between the wet and dry signals. Sync options from 4 to 1/64 bars. |
| 10 | `FB2` | Feedback2 | Feedback, channel 2 | continuous | channel-2 | 0.0 to 100 % | yes | [feedback](#glyph-feedback) | Allows feedback to be applied to the second channel. |
| 11 | `PN2` | Pan2 | Pan, channel 2 | continuous | channel-2 | -100.0 to 100 % | yes | [pan](#glyph-pan) | Allows panning of the second channel. |
| 12 | `HIC` | HiCut | High Cut | continuous |  | 2000 to 20000 k Hz | yes | [high cut](#glyph-high-cut) | Allows the high frequencies affected by the pitch shifting to be reduced. |

<a id="34-rotary-speaker"></a>

### Rotary Speaker (RotarySpkr)

<img src="diagrams/fx/34-rotary-speaker.svg" alt="Rotary Speaker front panel" width="720">

`FX Type` 34, creative, drawn with the rotary mark. 8 slots, drawn as knobs. Characters: [Vintage](#vintage), [Modulated](#modulated).

| Slot | Ref | Parameter | Reads as | Control | Group | Range | Mod | Glyph | Description |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `LOS` | LoSpeed | Low Speed | continuous |  | 0.1 to 4.0 Hz | yes | [rate](#glyph-rate) | Adjusts the rotational speed when the Low Speed operation is selected. |
| 2 | `HIS` | HiSpeed | High Speed | continuous |  | 2.0 to 9.9 Hz | yes | [rate](#glyph-rate) | Adjusts the rotational speed when the High Speed operation is selected. |
| 3 | `ACC` | Accel | Acceleration | continuous |  | 0.0 to 100 % | yes | [curve](#glyph-curve) | Adjusts how quickly the speed increases and decreases from the Slow mode to the Fast mode. |
| 4 | `DIS` | Distance | Distance | continuous |  | 0.0 to 100 % | yes | [size](#glyph-size) | Adjusts the distance between the Rotary speakers and the virtual microphone. |
| 5 | `BAL` | Balance | Balance | continuous |  | -100.0 to 100.0 | yes | [pan](#glyph-pan) | Adjusts the balance between the virtual horn and virtual drum controlling the signal tone. |
| 6 | `MIX` | Mix | Mix | continuous |  | 0.0 to 100 % | yes | [mix](#glyph-mix) | Controls the mix (or ratio) of wet (processed) and dry (unprocessed) signals. |
| 7 | `MOT` | Motor | Motor | switch |  | RUN, STOP | yes | [switch](#glyph-switch) | Allows the rotation effect of the motor to be disengaged (STOP). |
| 8 | `SPD` | Speed | Speed | selector |  | SLOW, FAST | yes | [rate](#glyph-rate) | SLOWFASTSelects either the slow or fast speeds for rotation. |

<!-- /generated:effects -->

## Corrections to the manual

Where the manual's effect tables contradict themselves, this is what was done
about it. Corrections to the rest of the specification are in
[the MIDI specification](midi-spec.md#corrections-to-the-manual).

<!-- generated:effect-corrections -->

| Effect | Slot | Parameter | Correction |
|---|---|---|---|
| [MoodFilter](#31-moog-type-filter) | 5 | Type | The options were read off the first three words of the description and lost Notch. The manual states four; the table cell states none. |
| [MoodFilter](#31-moog-type-filter) | 12 | 4-Pole | The manual's own table is malformed for this row: MIN reads "2P 4P" and MAX reads "100.0". Read as a two-state 2P to 4P, which is what the FX page screenshot beside the table shows, labelled POL and reading 4P. |

<!-- /generated:effect-corrections -->
