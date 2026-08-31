+++
id = "shell.example_shifted_vertical_axis"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_shifted_vertical_axis"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.15"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = x` and below by the *x*-axis over `[1, 2]` around the vertical line
`x = -1`.

## Solution

The shell radius is `r(x) = x - (-1) = x + 1`, height `h(x) = x`.

\[
V = \int_1^2 2\pi (x + 1)x\,dx = 2\pi \int_1^2 (x^2 + x)\,dx
  = 2\pi\left[\frac{x^3}{3} + \frac{x^2}{2}\right]_1^2 = \frac{23\pi}{3}
\]

## Hints

- The radius of a shell at position `x` is its distance to the axis of rotation `x = -1`, which is `x + 1`.
- Set up the integral with the shifted radius before evaluating.
