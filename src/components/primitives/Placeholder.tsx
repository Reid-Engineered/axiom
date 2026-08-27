import styles from "./Placeholder.module.css";

export interface PlaceholderProps {
  label: string;
  height?: string | number;
  width?: string | number;
  aspectRatio?: string;
  className?: string;
}

/**
 * Diagonal-striped element standing in for unrendered content or visualizers.
 */
export function Placeholder({
  label,
  height,
  width,
  aspectRatio,
  className = "",
}: PlaceholderProps) {
  const style = {
    ...(height !== undefined && { height: typeof height === "number" ? `${height}px` : height }),
    ...(width !== undefined && { width: typeof width === "number" ? `${width}px` : width }),
    ...(aspectRatio !== undefined && { aspectRatio }),
  };

  return (
    <div className={`${styles.placeholder} ${className}`} style={style}>
      <span className={styles.caption}>{label}</span>
    </div>
  );
}
