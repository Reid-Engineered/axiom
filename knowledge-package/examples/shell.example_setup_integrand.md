+++
id = "shell.example_setup_integrand"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis"]

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
label = "Example 2.15"
+++

## Problem

Consider the region bounded by `f(x) = x^3`, the *x*-axis, between `x = 0`
and `x = 1`, revolved around the line `x = -2`. Express the simplified
integrand (including `2*pi`) for finding the volume using the shell method —
do not evaluate the integral.

## Solution

The radius is `r(x) = x - (-2) = x + 2`, the height is `h(x) = x^3`.

\[
2\pi \cdot r(x) \cdot h(x) = 2\pi(x + 2)x^3 = 2\pi\left(x^4 + 2x^3\right)
\]

## Hints

- Identify the shell radius from the axis `x = -2` before multiplying by the height.
- Multiply out `(x + 2) \cdot x^3` to reach the simplified integrand.
