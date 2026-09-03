+++
id = "problem.shell_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height"]
difficulty = { min = 1, max = 2 }
generator = { id = "gen.shell_y_poly", version = 1 }
response_type = "symbolic-expression"
status = "verified"

[parameters.coeff]
type = "integer"
min = 2
max = 6
description = "Linear coefficient for quadratic curve f(x) = c*x - x^2"

[parameters.a]
type = "integer"
value = 0
description = "Left boundary of the interval"

[parameters.b]
type = "integer"
min = 1
max = { parameter = "coeff" }
description = "Right boundary; its inclusive maximum is the sampled coeff value"

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
+++

## Prompt

Define R as the region bounded above by the graph of f(x) = {coeff}x - x^2 and below by the
x-axis over the interval [{a}, {b}]. Find the volume of the solid of revolution formed by
revolving R around the y-axis.

## Solution

V = \int_{a}^{b} 2\pi x f(x) dx = 2\pi \int_{0}^{b} ({coeff}x^2 - x^3) dx
  = 2\pi [{coeff}x^3/3 - x^4/4]_{0}^{b}

## Hints

- Identify the shell radius and shell height as functions of x for rotation around the y-axis.
- For a region bounded by y = f(x) revolved around the y-axis, the shell radius is r(x) = x and the height is h(x) = {coeff}x - x^2.
- Set up the definite integral: V = \int_{0}^{{b}} 2\pi x ({coeff}x - x^2) dx.
- Evaluate the antiderivative 2\pi [{coeff}*x^3/3 - x^4/4] from 0 to {b}.
