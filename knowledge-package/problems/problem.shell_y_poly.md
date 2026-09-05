+++
id = "problem.shell_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = [
  "shell.setup_radius_height_y_axis",
  "shell.compute_volume_y_axis_single_curve",
]
difficulty = { min = 1, max = 2 }
generator = { id = "gen.shell_y_poly", version = 1 }
response_type = "symbolic-expression"
status = "verified"

[parameters.coeff]
type = "integer"
min = 2
max = 6
description = "Linear coefficient c of the quadratic curve f(x) = c*x - x^2"

[parameters.a]
type = "integer"
value = 0
description = "Left boundary of the interval; fixed at the origin so the shell radius r(x) = x stays non-negative"

[parameters.b]
type = "integer"
min = 1
max = { parameter = "coeff" }
description = "Right boundary; its inclusive maximum is the sampled coeff, which keeps f(x) = c*x - x^2 non-negative across [0, b]"

[canonical_solution]
expression = "2*pi*(coeff*b^3/3 - b^4/4)"

[[hints]]
level = 1

[[hints]]
level = 2

[[hints]]
level = 3

[[hints]]
level = 4

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

## Prompt

Define R as the region bounded above by the graph of f(x) = {coeff}x - x^2 and below by the
x-axis over the interval [{a}, {b}]. Find the volume of the solid of revolution formed by
revolving R around the y-axis.

## Solution

Rotation is about the y-axis and the region is described as a function of x, so the shell
radius is r(x) = x and the shell height is h(x) = c*x - x^2, where c is the sampled linear
coefficient. Rule 2.6 then gives

V = \int_0^b 2\pi\, r(x)\, h(x)\,dx
  = \int_0^b 2\pi x\,(c x - x^2)\,dx
  = 2\pi \int_0^b (c x^2 - x^3)\,dx
  = 2\pi \left[ \frac{c x^3}{3} - \frac{x^4}{4} \right]_0^b
  = 2\pi \left( \frac{c b^3}{3} - \frac{b^4}{4} \right).

Because b \le c, the height c x - x^2 = x(c - x) is non-negative across the whole interval,
so this is the volume of a genuine solid of revolution rather than a signed integral.

## Hints

- Rotation is around the y-axis and the region is given as a function of x, so identify the shell radius and the shell height as functions of x.
- The shell radius is the distance from the axis of rotation, r(x) = x, and the shell height is the curve itself, h(x) = {coeff}x - x^2.
- Assemble the definite integral over the given interval: V = \int_{0}^{{b}} 2\pi x ({coeff}x - x^2) dx.
- Expand the integrand to 2\pi({coeff}x^2 - x^3) and evaluate its antiderivative 2\pi[{coeff}x^3/3 - x^4/4] from 0 to {b}.
