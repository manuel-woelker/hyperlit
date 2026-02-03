/** @jsxImportSource @emotion/react */
import { Global, css } from '@emotion/react'
import styled from '@emotion/styled'
import { useEffect, useState } from 'react'
import { getSiteInfo, type SiteInfo } from './api/client.ts'
import { setupSSE } from './api/sse.ts'
import SearchPage from './pages/SearchPage.tsx'
import DocumentPage from './pages/DocumentPage.tsx'
import SplitLayout from './components/SplitLayout.tsx'
import EmptyDocumentPlaceholder from './components/EmptyDocumentPlaceholder.tsx'

/* 📖 # Why use light and airy colors for documentation?
Documentation requires extended reading sessions, which demands a light, low-contrast
visual design that reduces eye strain. The chosen palette uses:

1. **Soft backgrounds**: Off-white (#fafbfc) rather than stark white reduces glare
2. **Gentle borders**: Light grays (#e8eaed) provide structure without harsh lines
3. **Subtle shadows**: Soft, diffuse shadows create depth without visual weight
4. **Muted accents**: Pastel tones for badges and highlights that don't distract

This approach prioritizes content readability over visual flair, ensuring developers
can focus on understanding the documentation rather than navigating the interface.
*/

/* 📖 # Why a sticky header with search?
A sticky header keeps the search functionality accessible at all times as users
scroll through documentation. This design pattern provides:

1. **Persistent access**: Users can search from anywhere without scrolling back up
2. **Context preservation**: The site title and search remain visible for orientation
3. **Efficient workflow**: Quick searches while reading deep in a document
4. **Mobile-friendly**: Essential for long-scrolling content on smaller screens
*/

const Container = styled.div`
  min-height: 100vh;
  display: flex;
  flex-direction: column;
`

const Header = styled.header`
  position: sticky;
  top: 0;
  z-index: 100;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(8px);
  color: #4a5568;
  padding: 1rem 2rem;
  border-bottom: 1px solid #e8eaed;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
`

const HeaderContent = styled.div`
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  align-items: center;
  gap: 2rem;
`

const Title = styled.h1`
  margin: 0;
  font-size: 1.25rem;
  font-weight: 500;
  color: #2d3748;
  cursor: pointer;
  letter-spacing: -0.02em;
  white-space: nowrap;
  
  &:hover {
    color: #4a5568;
  }
`

const SearchContainer = styled.div`
  flex: 1;
  max-width: 500px;
`

const SearchInput = styled.input`
  width: 100%;
  padding: 0.625rem 1rem;
  font-size: 0.9375rem;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  outline: none;
  transition: all 0.2s ease;
  background: #f7fafc;
  color: #4a5568;

  &:focus {
    border-color: #a0aec0;
    box-shadow: 0 0 0 3px rgba(160, 174, 192, 0.1);
    background: #ffffff;
  }

  &::placeholder {
    color: #a0aec0;
  }
`

const VersionTag = styled.span`
  font-size: 0.875rem;
  color: #a0aec0;
  white-space: nowrap;
`

const Main = styled.main`
  flex: 1;
  display: flex;
  overflow: hidden;
`

const globalStyles = css`
  * {
    box-sizing: border-box;
  }
  
  html, body {
    margin: 0;
    padding: 0;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    background: #fafbfc;
    color: #4a5568;
  }
  
  body {
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif !important;
    line-height: 1.6;
  }
`

function App() {
  const [siteInfo, setSiteInfo] = useState<SiteInfo | null>(null)
  const [hash, setHash] = useState(window.location.hash)
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedDocId, setSelectedDocId] = useState<string | null>(null)
  const [reloadKey, setReloadKey] = useState(0)

  useEffect(() => {
    const handleHashChange = () => setHash(window.location.hash)
    window.addEventListener('hashchange', handleHashChange)
    return () => window.removeEventListener('hashchange', handleHashChange)
  }, [])

  useEffect(() => {
    getSiteInfo()
      .then(setSiteInfo)
      .catch(console.error)
  }, [])

  /* 📖 # Why force React component reload instead of full page refresh?
  When documentation files change, we want to update the UI without losing scroll
  position or causing a jarring page flash. By incrementing a key on the container,
  React unmounts and remounts all child components, which triggers their useEffect
  hooks to refetch data. This preserves scroll position and provides a smoother UX
  than window.location.reload().
  */
  // Set up hot-reload via Server-Sent Events
  useEffect(() => {
    const cleanup = setupSSE(() => {
      // Force React to remount all components by changing the key
      setReloadKey(prev => prev + 1)
    })

    return cleanup
  }, [])

  /* 📖 # Why parse both #/search?doc={id} and #/doc/{id} formats?
  We support two URL formats for backward compatibility:
  - New format: #/search?doc={id} represents the split view with search and document
  - Old format: #/doc/{id} redirects to split view for backward compatibility

  This ensures existing bookmarks and links continue to work while transitioning
  to the new split-pane architecture.
  */
  // Parse route from hash and update state
  useEffect(() => {
    const path = hash.replace('#', '') || '/search'

    if (path.startsWith('/doc/')) {
      // Old format: redirect to split view
      const encodedDocId = path.replace('/doc/', '')
      const docId = decodeURIComponent(encodedDocId)
      setSelectedDocId(docId)
      // Update hash to new format without triggering hashchange
      window.history.replaceState(null, '', `#/search?doc=${encodeURIComponent(docId)}`)
    } else if (path.startsWith('/search')) {
      // New format: parse doc parameter from query string
      const queryStart = path.indexOf('?')
      if (queryStart !== -1) {
        const queryString = path.slice(queryStart + 1)
        const params = new URLSearchParams(queryString)
        const docId = params.get('doc')
        setSelectedDocId(docId)
      } else {
        setSelectedDocId(null)
      }
    }
  }, [hash])

  const goToDocument = (id: string) => {
    setSelectedDocId(id)
    window.location.hash = `#/search?doc=${encodeURIComponent(id)}`
  }

  const closeDocument = () => {
    setSelectedDocId(null)
    window.location.hash = '#/search'
  }

  const goToSearch = () => {
    closeDocument()
  }

  return (
    <>
      <Global styles={globalStyles} />
      <Container>
        <Header>
          <HeaderContent>
            <Title onClick={goToSearch}>
              {siteInfo?.title || 'Hyperlit Documentation'}
            </Title>
            <SearchContainer>
              <SearchInput
                type="text"
                placeholder="Search documentation..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </SearchContainer>
            {siteInfo?.version && (
              <VersionTag>v{siteInfo.version}</VersionTag>
            )}
          </HeaderContent>
        </Header>
        <Main key={reloadKey}>
          <SplitLayout
            leftPanel={
              <SearchPage
                query={searchQuery}
                onDocumentClick={goToDocument}
                selectedDocId={selectedDocId}
              />
            }
            rightPanel={
              selectedDocId ? (
                <DocumentPage
                  documentId={selectedDocId}
                  onClose={closeDocument}
                />
              ) : (
                <EmptyDocumentPlaceholder />
              )
            }
          />
        </Main>
      </Container>
    </>
  )
}

export default App
