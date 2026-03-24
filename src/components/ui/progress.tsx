"use client"

import * as React from "react"
import * as ProgressPrimitive from "@radix-ui/react-progress"

import { cn } from "@/lib/utils"

/**
 * Progress Component
 * 
 * Displays a horizontal progress bar with proper accessibility attributes.
 * The progress value should be between 0 and 100.
 * 
 * @example
 * ```tsx
 * <Progress value={75} aria-label="Upload progress" />
 * ```
 */
function Progress({
  className,
  value,
  ...props
}: React.ComponentProps<typeof ProgressPrimitive.Root>) {
  // Ensure value is clamped between 0 and 100
  const normalizedValue = Math.min(100, Math.max(0, value || 0))
  
  return (
    <ProgressPrimitive.Root
      data-slot="progress"
      className={cn(
        "bg-primary/20 relative h-2 w-full overflow-hidden rounded-full",
        className
      )}
      // Accessibility: These attributes are essential for screen readers
      // to understand the progress state
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={normalizedValue}
      aria-valuetext={`${normalizedValue}%`}
      {...props}
    >
      <ProgressPrimitive.Indicator
        data-slot="progress-indicator"
        className="bg-primary h-full w-full flex-1 transition-all"
        style={{ transform: `translateX(-${100 - normalizedValue}%)` }}
      />
    </ProgressPrimitive.Root>
  )
}

export { Progress }
