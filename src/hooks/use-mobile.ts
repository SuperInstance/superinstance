/**
 * useIsMobile Hook
 * 
 * Detects whether the current viewport is mobile-sized (<768px).
 * Uses media query listener for responsive updates.
 * 
 * @returns {boolean} true if viewport width is less than 768px, false otherwise
 * 
 * @example
 * ```tsx
 * function MyComponent() {
 *   const isMobile = useIsMobile()
 *   return (
 *     <div className={isMobile ? 'p-2' : 'p-6'}>
 *       {isMobile ? 'Mobile view' : 'Desktop view'}
 *     </div>
 *   )
 * }
 * ```
 * 
 * @remarks
 * - Returns `false` during SSR (server-side rendering)
 * - Updates automatically on window resize
 * - Uses matchMedia for efficient change detection
 */
import * as React from "react"

/** Breakpoint width in pixels for mobile detection */
const MOBILE_BREAKPOINT = 768

export function useIsMobile() {
  // Use undefined initial state to handle SSR gracefully
  const [isMobile, setIsMobile] = React.useState<boolean | undefined>(undefined)

  React.useEffect(() => {
    // Create media query listener for efficient resize detection
    const mql = window.matchMedia(`(max-width: ${MOBILE_BREAKPOINT - 1}px)`)
    
    const onChange = () => {
      setIsMobile(window.innerWidth < MOBILE_BREAKPOINT)
    }
    
    // Subscribe to changes
    mql.addEventListener("change", onChange)
    
    // Set initial value
    setIsMobile(window.innerWidth < MOBILE_BREAKPOINT)
    
    // Cleanup on unmount
    return () => mql.removeEventListener("change", onChange)
  }, [])

  // Coerce undefined to false for SSR safety
  return !!isMobile
}
