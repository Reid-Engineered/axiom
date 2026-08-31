+++
id = "shell.example_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_y_axis_single_curve"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.13"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 4x - x^2` and below by the *x*-axis over `[0, 3]` around the *y*-axis.

## Solution

The shell radius is `r(x) = x`, height `h(x) = 4x - x^2`.

\[
V = \int_0^3 2\pi x(4x - x^2)\,dx = 2\pi\left[\frac{4x^3}{3} - \frac{x^4}{4}\right]_0^3
  = 2\pi\left(36 - \frac{81}{4}\right) = \frac{63\pi}{2}
\]

## Hints

- Identify the shell radius and height as functions of `x` for rotation around the *y*-axis.
- The shell radius is `r(x) = x` and the height is `h(x) = 4x - x^2`.
