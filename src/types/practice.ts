export type ResponseType = 'symbolic-expression' | 'numeric';

export interface Attempt {
  attemptId: string;
  prompt: string;
  responseType: ResponseType;
  hintsTotal: number;
}

export type ResponseValue =
  | { responseType: 'symbolic-expression'; value: string }
  | { responseType: 'numeric'; value: number };

export type AttemptStatus = 'open' | 'solved';

export interface EvaluationResult {
  correct: boolean;
  status: AttemptStatus;
  submissionCount: number;
}

export interface Hint {
  hintText: string;
  hintsRevealed: number;
  hintsTotal: number;
}

/** An attempt's current learner-facing state, the single hydration path for a session. */
export interface AttemptDescription {
  prompt: string;
  responseType: ResponseType;
  hintsTotal: number;
  hintsRevealed: number;
  status: AttemptStatus;
  submissionCount: number;
}

