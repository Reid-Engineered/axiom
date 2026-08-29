import type { VisualizationScene } from '../types';

export const shellMethodScene: VisualizationScene = {
  id: 'scene-shells-about-zero',
  name: 'Shells about x = 0',
  coordinateSystem: { kind: 'coordinate-system', dimensions: 3, axisLabels: ['x', 'y', 'z'] },
  functions: [
    {
      kind: 'function',
      id: 'function-parabola',
      expression: 'y = x² − 1',
      variable: 'x',
      domain: [1, 3],
    },
  ],
  regions: [
    {
      kind: 'region',
      id: 'region-under-parabola',
      functionId: 'function-parabola',
      lowerBound: 1,
      upperBound: 3,
      baseline: 0,
    },
  ],
  axes: [{ kind: 'axis', id: 'axis-x-zero', orientation: 'vertical', value: 0, label: 'x = 0' }],
  revolutions: [
    {
      kind: 'revolution',
      regionId: 'region-under-parabola',
      axisId: 'axis-x-zero',
      angleRadians: Math.PI * 2,
    },
  ],
  shells: [
    {
      kind: 'shell',
      id: 'shell-selected',
      position: 2.4,
      radius: 2.4,
      height: 4.76,
      thickness: 0.025,
      approximateVolume: 4.48,
    },
  ],
  annotations: [
    {
      kind: 'annotation',
      id: 'annotation-radius',
      text: 'radius is measured from the axis',
      targetPrimitiveId: 'shell-selected',
    },
  ],
};
