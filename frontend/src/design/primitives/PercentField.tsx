import { NumberField, type NumberFieldProps } from "./NumberField";

export interface PercentFieldProps extends Omit<NumberFieldProps, "formatOptions" | "unit" | "step"> {
  /** Decimals shown (default 1, frontend.md §8). */
  digits?: number;
}

/**
 * A percent input over a fraction in model units (0.3 = 30 %; server-api.md §2): the value and the min/max stay
 * fractions, the display is a percent with one decimal.
 */
export function PercentField({ digits = 1, ...rest }: PercentFieldProps) {
  return (
    <NumberField
      {...rest}
      step={10 ** -(digits + 2)}
      formatOptions={{ style: "percent", minimumFractionDigits: digits, maximumFractionDigits: digits }}
    />
  );
}
