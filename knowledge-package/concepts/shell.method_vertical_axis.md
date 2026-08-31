+++
id = "shell.method_vertical_axis"
name = "The Method of Cylindrical Shells (Vertical Axis of Revolution)"
topic = "2.3 Volumes of Revolution: Cylindrical Shells"
prerequisite_ids = []
related_ids = ["shell.method_horizontal_axis"]

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
+++

A method for calculating the volume of a solid of revolution by decomposing the
region into representative vertical cylindrical shells and integrating with
respect to `x`. For rotation around the *y*-axis:

\[
V = \int_a^b 2\pi x f(x)\,dx
\]

between two curves, `h(x) = f(x) - g(x)` replaces `f(x)`. For rotation around a
vertical line `x = k`, the radius is adjusted to `|x - k|`.
