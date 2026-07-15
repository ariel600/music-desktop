import * as React from "react";
import * as SliderPrimitive from "@radix-ui/react-slider";
import { cn } from "@/lib/utils";

const Slider = React.forwardRef<
  React.ComponentRef<typeof SliderPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof SliderPrimitive.Root>
>(({ className, orientation = "horizontal", ...props }, ref) => (
  <SliderPrimitive.Root
    ref={ref}
    orientation={orientation}
    className={cn(
      "relative flex touch-none select-none items-center",
      orientation === "vertical"
        ? "h-28 w-5 flex-col justify-end"
        : "w-full",
      className,
    )}
    {...props}
  >
    <SliderPrimitive.Track
      className={cn(
        "relative grow overflow-hidden rounded-full bg-teal-100",
        orientation === "vertical" ? "w-2 h-full" : "h-2 w-full",
      )}
    >
      <SliderPrimitive.Range className="absolute bg-teal-600 rounded-full data-[orientation=vertical]:w-full data-[orientation=horizontal]:h-full" />
    </SliderPrimitive.Track>
    <SliderPrimitive.Thumb className="block h-4 w-4 rounded-full border-2 border-teal-700 bg-white shadow focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-teal-500" />
  </SliderPrimitive.Root>
));
Slider.displayName = SliderPrimitive.Root.displayName;

export { Slider };
