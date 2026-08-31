+++
id = "shell.example_y_reciprocal"
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
label = "Example 2.12"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 3/x` and below by the *x*-axis over `[1, 4]` around the *y*-axis.

## Solution

The shell radius is `r(x) = x`, height `h(x) = 3/x`; the integrand simplifies
to a constant.

\[
V = \int_1^4 2\pi x \left(\frac{3}{x}\right) dx = 2\pi \int_1^4 3\,dx
  = 2\pi \cdot 3 \cdot (4 - 1) = 18\pi
\]

## Hints

- Write the shell volume formula and notice that `x` times `f(x)` simplifies to a constant.
- Integrating a constant over `[1, 4]` is just the constant times the interval length.
