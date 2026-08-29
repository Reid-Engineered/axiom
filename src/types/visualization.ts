export interface CoordinateSystemPrimitive {
  kind: 'coordinate-system';
  dimensions: 3;
  axisLabels: [string, string, string];
}

export interface FunctionPrimitive {
  kind: 'function';
  id: string;
  expression: string;
  variable: string;
  domain: [number, number];
}

export interface RegionPrimitive {
  kind: 'region';
  id: string;
  functionId: string;
  lowerBound: number;
  upperBound: number;
  baseline: number;
}

export interface AxisPrimitive {
  kind: 'axis';
  id: string;
  orientation: 'horizontal' | 'vertical';
  value: number;
  label: string;
}

export interface RevolutionPrimitive {
  kind: 'revolution';
  regionId: string;
  axisId: string;
  angleRadians: number;
}

export interface ShellPrimitive {
  kind: 'shell';
  id: string;
  position: number;
  radius: number;
  height: number;
  thickness: number;
  approximateVolume: number;
}

export interface AnnotationPrimitive {
  kind: 'annotation';
  id: string;
  text: string;
  targetPrimitiveId: string;
}

/** Verified primitives consumed by the inert placeholder now and the Stage 8 renderer later. */
export interface VisualizationScene {
  id: string;
  name: string;
  coordinateSystem: CoordinateSystemPrimitive;
  functions: FunctionPrimitive[];
  regions: RegionPrimitive[];
  axes: AxisPrimitive[];
  revolutions: RevolutionPrimitive[];
  shells: ShellPrimitive[];
  annotations: AnnotationPrimitive[];
}
