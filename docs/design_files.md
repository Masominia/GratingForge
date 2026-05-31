# Design files

One statement per line. `#` starts a comment; the comment lines at the top of a file are its description and appear in the report. Lengths are in micrometres, angles in degrees. Everything has a default, so an empty file is a valid design: a 220 nm silicon-on-insulator grating with a 70 nm etch, emitting at 10 degrees into oxide at 1.55 um.

## Statements

| statement | meaning | default |
|---|---|---|
| `title <text>` | name used in reports | empty |
| `wavelength <um>` | design wavelength | 1.55 |
| `stack core=<um> box=<um>` | core and buried-oxide thickness | 0.22, 2.0 |
| `materials core= clad= box= substrate=` | `si`, `sio2`, `sin`, `air` or a constant index | si, sio2, sio2, si |
| `polarisation te` or `tm` | polarisation of the slab mode | te |
| `grating period=<um>` or `angle=<deg>` | give one; the other is computed | angle=10 |
| `grating fill= etch=<um> length=<um>` | unetched fraction, etch depth, grating length | 0.5, 0.07, 15 |
| `leak <per um>` | power leakage rate; the upper limit for `apodise gaussian` | 0.14 |
| `apodise uniform` | constant leakage | yes |
| `apodise linear from= to=` | leakage ramp along the grating | |
| `apodise gaussian` | leakage profile that emits a Gaussian matched to the fibre | |
| `up <fraction>` | upward share of the radiated power, grating alone | 0.5 |
| `substrate on` or `off` | include the reflection from the substrate | off |
| `fibre mfd=<um>` | mode field diameter | 10.4 |
| `fibre angle=<deg>` | fibre tilt; left out, the fibre follows the emission angle | follows |
| `fibre position=<um>` | fibre centre along the grating; left out, it is optimised | optimised |
| `sweep <from um> <to um> <points>` | wavelength range of the spectrum | 1.5 1.6 201 |
| `expect <kind> <args>` | a check, see below | |

The etch may equal the core thickness (a fully etched grating) but not exceed it. The fill must lie strictly between 0 and 1.

## Expectations

| kind | arguments | passes when |
|---|---|---|
| `n_full`, `n_etched`, `n_grating`, `group_index` | value, tolerance | the index is within the tolerance |
| `period` | um, tolerance | |
| `angle` | degrees, tolerance | emission angle in the cladding |
| `orders` | count | that many orders radiate into the cladding |
| `second_order` | um, tolerance | wavelength of vertical emission |
| `dtheta_dlambda` | degrees per nm, relative tolerance | |
| `radiated` | fraction, tolerance | share of guided power leaving the grating |
| `up` | fraction, tolerance | upward share, substrate included if on |
| `overlap` | fraction, tolerance | overlap of the emitted field with the fibre mode |
| `efficiency`, `efficiency_db` | value, tolerance | guided power to fibre |
| `position` | um, tolerance | fibre position |
| `peak_wavelength` | um, tolerance | |
| `bandwidth_1db`, `bandwidth_3db` | nm, relative tolerance | |
| `bandwidth_formula` | relative tolerance | the 1 dB width agrees with the Gaussian-beam formula |

A malformed expectation is reported as a failed check with its line number; the other checks still run.

## Output

`gratingforge check <file>` prints `key value` lines and one `check PASS|FAIL <name> :: <detail>` line per expectation. Exit status: 0 if all checks pass, 1 if one fails, 2 if the file cannot be read or the design is impossible (no guided mode, first order not radiating). `gratingforge run <file>` adds `field <x> <amplitude> <leakage>` and `spectrum <wavelength> <efficiency>` lines.

## Example

    # A directional apodised coupler on a 2 um buried oxide.
    # Checked by hand: the angle must be the one requested.
    title Example
    grating angle=10 length=30
    leak 0.5
    apodise gaussian
    up 0.7
    substrate on
    expect angle 10 1e-6
    expect orders 1
