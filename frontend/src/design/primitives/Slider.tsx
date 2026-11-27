import { Label, Slider as AriaSlider, SliderOutput, SliderThumb, SliderTrack, type SliderProps as AriaSliderProps } from "react-aria-components";
import { cx } from "./cx";

export interface SliderProps extends Omit<AriaSliderProps<number>, "className" | "children"> {
  label: string;
  className?: string;
}

/** A single-value slider with its output (React Aria): arrow keys, Page Up/Down, Home/End. */
export function Slider({ label, className, ...rest }: SliderProps) {
  return (
    <AriaSlider {...rest} className={cx("flex flex-col gap-1", className)}>
      <div className="flex justify-between text-xs">
        <Label className="text-fg-2">{label}</Label>
        <SliderOutput className="num text-fg-1" />
      </div>
      <SliderTrack className="relative h-6 w-full">
        {({ state }) => (
          <>
            <div className="absolute top-1/2 h-1 w-full -translate-y-1/2 rounded-full bg-surface-3" />
            <div
              className="absolute top-1/2 h-1 -translate-y-1/2 rounded-full bg-accent-fill"
              style={{ width: `${state.getThumbPercent(0) * 100}%` }}
            />
            <SliderThumb className="top-1/2 size-4 rounded-full border-2 border-accent-fill bg-surface-1 data-[focus-visible]:outline-2 data-[focus-visible]:outline-offset-2 data-[focus-visible]:outline-accent" />
          </>
        )}
      </SliderTrack>
    </AriaSlider>
  );
}
