import assert from 'node:assert/strict';

// Read only the learner-visible prompt; never use the provider's canonical answer or IPC.
export function shellVolumeAnswer(prompt) {
  const match = prompt
    .replace(/\s+/g, ' ')
    .match(/f\(x\) = (\d+)x - x\^2.*interval \[0, (\d+)\].*revolving R around the y-axis\./);
  assert.ok(match, `Expected a generated shell_y_poly prompt, received: ${prompt}`);
  const [, coefficient, bound] = match.map(Number);
  assert.ok(coefficient >= 2 && coefficient <= 6 && bound >= 1 && bound <= coefficient);
  // Integral of 2*pi*x*(c*x-x^2), simplified independently to a rational multiple of pi.
  return `${bound ** 3 * (4 * coefficient - 3 * bound)}*pi/6`;
}
