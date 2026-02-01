/** @jsxImportSource @emotion/react */
import { Global, css } from '@emotion/react'
import styled from '@emotion/styled'
import { useEffect, useState } from 'react'
import { getSiteInfo, type SiteInfo } from './api/client.ts'
import SearchPage from './pages/SearchPage.tsx'
import DocumentPage from './pages/DocumentPage.tsx'

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
  max-width: 1200px;
  width: 100%;
  margin: 0 auto;
  padding: 2rem;
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
  const [previousSearchQuery, setPreviousSearchQuery] = useState('')

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

  // Parse route from hash
  const getRoute = () => {
    const path = hash.replace('#', '') || '/search'
    
    if (path.startsWith('/doc/')) {
      // 📖 # Why decode the document ID from the hash?
      // The document ID was encoded with encodeURIComponent() when navigating
      // (in goToDocument), so we need to decode it here to get the original ID.
      // Without decoding, the ID would be double-encoded when the API client
      // calls encodeURIComponent() again before making the request.
      const encodedDocId = path.replace('/doc/', '')
      const docId = decodeURIComponent(encodedDocId)
      return { type: 'document', docId }
    }
    
    return { type: 'search' }
  }

  const route = getRoute()

  const goToSearch = () => {
    // Restore the previous search query when going back to search
    setSearchQuery(previousSearchQuery)
    window.location.hash = '#/search'
  }

  const goToDocument = (id: string) => {
    // Save current search query before navigating to document
    setPreviousSearchQuery(searchQuery)
    setSearchQuery('')
    window.location.hash = `#/doc/${encodeURIComponent(id)}`
  }

  // Navigate to search when query is entered while on document page
  useEffect(() => {
    if (route.type === 'document' && searchQuery.trim()) {
      goToSearch()
    }
  }, [searchQuery, route.type])

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
        <Main>
          {route.type === 'search' && (
            <SearchPage 
              query={searchQuery}
              onDocumentClick={goToDocument} 
            />
          )}
          {route.type === 'document' && route.docId && (
            <DocumentPage 
              documentId={route.docId} 
              onBack={goToSearch}
            />
          )}
        </Main>
      </Container>
    </>
  )
}

export default App
