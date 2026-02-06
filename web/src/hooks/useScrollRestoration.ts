import { useEffect, useLayoutEffect, useRef } from 'react'

/* 📖 # Why use module-level in-memory storage for scroll preservation during HMR?

When Vite hot module replacement (HMR) triggers component remounts, the DOM
is destroyed and recreated, losing scroll positions. We need a storage mechanism
that survives React component lifecycle events during hot reload but resets on
full page reload.

Module-level Map was chosen because:
1. **Survives remounts**: Persists across component unmounts during HMR
2. **HMR-safe**: Vite preserves module state during hot updates
3. **Memory-only**: Cleared on page reload, doesn't pollute browser storage
4. **Simple**: No serialization overhead, direct number storage

Alternative approaches considered:
- sessionStorage: Over-persistent (survives page reloads when we want fresh state)
- Modify remount strategy: Would require major architectural changes
- Query selectors: Fragile and timing-dependent
*/

// Module-level storage survives component remounts but resets on page reload
const scrollPositions = new Map<string, number>()

// Preserve scroll positions across HMR updates
if (import.meta.hot) {
  if (import.meta.hot.data.scrollPositions) {
    // Restore from previous HMR state
    const saved = import.meta.hot.data.scrollPositions as Map<string, number>
    saved.forEach((value, key) => scrollPositions.set(key, value))
  }

  import.meta.hot.dispose(() => {
    // Save for next HMR update
    import.meta.hot!.data.scrollPositions = scrollPositions
  })
}

/**
 * Preserves scroll position across component remounts (e.g., during HMR).
 *
 * @param paneId - Unique identifier for the scrollable pane
 * @returns Ref to attach to the scrollable container element
 */
export default function useScrollRestoration(paneId: string) {
  const ref = useRef<HTMLDivElement>(null)
  const key = `hyperlit-scroll-${paneId}`

  // Restore scroll position on mount
  useLayoutEffect(() => {
    const saved = scrollPositions.get(key)
    if (saved !== undefined) {
      const position = saved
      // Double RAF ensures DOM is fully ready and content is rendered
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          ref.current?.scrollTo(0, position)
        })
      })
    }
  }, [paneId, key])

  // Save scroll position on scroll (debounced) and unmount
  useEffect(() => {
    const element = ref.current
    if (!element) return

    let timeoutId: number | null = null

    const handleScroll = () => {
      if (timeoutId !== null) clearTimeout(timeoutId)
      timeoutId = window.setTimeout(() => {
        scrollPositions.set(key, element.scrollTop)
      }, 100) // Reduced debounce for faster capture
    }

    element.addEventListener('scroll', handleScroll, { passive: true })

    return () => {
      element.removeEventListener('scroll', handleScroll)
      // Execute pending save before cleanup
      if (timeoutId !== null) {
        clearTimeout(timeoutId)
        // Only save if scrollTop is non-zero (actual scrolled content)
        if (element.scrollTop > 0) {
          scrollPositions.set(key, element.scrollTop)
        }
      }
      // Don't save zero positions - they indicate content being reset
    }
  }, [paneId, key])

  return ref
}
