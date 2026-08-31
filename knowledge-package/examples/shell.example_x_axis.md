+++
id = "shell.example_x_axis"
concept_id = "shell.method_horizontal_axis"
objective_ids = ["shell.compute_volume_x_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.14"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded on the
right by `x = 2*sqrt(y)` and on the left by the *y*-axis, for `y` in `[0, 4]`,
around the *x*-axis.

## Solution

The shell radius is `r(y) = y`, height `h(y) = 2*sqrt(y)`.

\[
V = \int_0^4 2\pi y \left(2\sqrt{y}\right) dy = 4\pi \int_0^4 y^{3/2}\,dy
  = 4\pi \left[\frac{2}{5}y^{5/2}\right]_0^4 = \frac{256\pi}{5}
\]

## Hints

- When revolving around the *x*-axis with shells, the variable of integration is `y`.
- The shell radius is `r(y) = y` and the height is the horizontal extent `x = g(y)`.
