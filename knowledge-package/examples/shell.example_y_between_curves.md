+++
id = "shell.example_y_between_curves"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_y_axis_between_curves"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "derived"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.16"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 2x` and below by `g(x) = x^2` around the *y*-axis.

## Solution

The curves intersect at `x = 0` and `x = 2`. The shell height is
`h(x) = 2x - x^2`.

\[
V = \int_0^2 2\pi x(2x - x^2)\,dx = 2\pi\left[\frac{2x^3}{3} - \frac{x^4}{4}\right]_0^2
  = 2\pi\left(\frac{16}{3} - 4\right) = \frac{8\pi}{3}
\]

## Hints

- Find the *x*-limits of integration by setting the two functions equal.
- The shell height is the difference between the upper and lower curves.
