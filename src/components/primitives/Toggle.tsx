import styles from "./Toggle.module.css";

export interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  label?: string;
  "aria-label"?: string;
  className?: string;
}

/**
 * 30x18 toggle switch for settings and tool enable/disable states.
 */
export function Toggle({
  checked,
  onChange,
  disabled = false,
  label,
  "aria-label": ariaLabel,
  className = "",
}: ToggleProps) {
  const handleClick = () => {
    if (!disabled) {
      onChange(!checked);
    }
  };

  return (
    <label
      className={`${styles.container} ${disabled ? styles.disabled : ""} ${className}`}
      onClick={handleClick}
    >
      <span
        role="switch"
        aria-checked={checked}
        aria-label={ariaLabel || label}
        tabIndex={disabled ? -1 : 0}
        className={`${styles.track} ${checked ? styles.trackChecked : ""}`}
        onKeyDown={(e) => {
          if (e.key === " " || e.key === "Enter") {
            e.preventDefault();
            handleClick();
          }
        }}
      >
        <span
          className={`${styles.knob} ${checked ? styles.knobChecked : ""}`}
        />
      </span>
      {label && <span>{label}</span>}
    </label>
  );
}
