/** @jsxImportSource @emotion/react */
import styled from '@emotion/styled'
import type { ReactNode } from 'react'
import useScrollRestoration from '../hooks/useScrollRestoration'

/* 📖 # Why a fixed 40/60 split instead of resizable panes?
A fixed split ratio provides several benefits:
1. **Consistent experience**: Users always know where to look for content
2. **Simpler implementation**: No drag handles, resize logic, or state management
3. **Optimal proportions**: 40% gives enough room for result scanning, 60% for comfortable reading
4. **Faster initial development**: Resizable panes can be added later if user feedback demands it

The 40/60 split was chosen based on the content needs: search results are scannable
(titles + previews), while documentation requires more horizontal space for comfortable
reading of paragraphs and code blocks.
*/

const SplitContainer = styled.div`
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
`

const LeftPanel = styled.div<{ hasRightPanel: boolean }>`
  width: ${props => props.hasRightPanel ? '40%' : '100%'};
  min-width: ${props => props.hasRightPanel ? '300px' : '0'};
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
`

const Divider = styled.div`
  width: 1px;
  background: #e8eaed;
  flex-shrink: 0;
`

const RightPanel = styled.div`
  flex: 1;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
`

interface SplitLayoutProps {
  leftPanel: ReactNode
  rightPanel: ReactNode | null
}

export default function SplitLayout({ leftPanel, rightPanel }: SplitLayoutProps) {
  const leftPanelRef = useScrollRestoration('left')
  const rightPanelRef = useScrollRestoration('right')

  return (
    <SplitContainer>
      <LeftPanel ref={leftPanelRef} hasRightPanel={!!rightPanel}>
        {leftPanel}
      </LeftPanel>
      {rightPanel && (
        <>
          <Divider />
          <RightPanel ref={rightPanelRef}>
            {rightPanel}
          </RightPanel>
        </>
      )}
    </SplitContainer>
  )
}
