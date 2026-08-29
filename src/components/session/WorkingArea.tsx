/** Dashed "YOUR WORKING" area containing the learner's own lines (screen 5). */
export interface WorkingAreaProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

export function WorkingArea({ value, onChange, placeholder, className = '' }: WorkingAreaProps) {
  return (
    <label className={`${styles.area} ${className}`}>
      <span>Your working</span>
      <textarea
        aria-label="Your working"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
      />
    </label>
  );
}
import styles from './WorkingArea.module.css';
