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

const Container = styled.div`
  min-height: 100vh;
  display: flex;
  flex-direction: column;
`

const Header = styled.header`
  background: #ffffff;
  color: #4a5568;
  padding: 1.25rem 2rem;
  border-bottom: 1px solid #e8eaed;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
`

const HeaderContent = styled.div`
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
`

const Title = styled.h1`
  margin: 0;
  font-size: 1.5rem;
  font-weight: 500;
  color: #2d3748;
  cursor: pointer;
  letter-spacing: -0.02em;
  
  &:hover {
    color: #4a5568;
  }
`

const Main = styled.main`
  flex: 1;
  max-width: 1200px;
  width: 100%;
  margin: 0 auto;
  padding: 2.5rem 2rem;
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
      const docId = path.replace('/doc/', '')
      return { type: 'document', docId }
    }
    
    return { type: 'search' }
  }

  const route = getRoute()

  const goToSearch = () => {
    window.location.hash = '#/search'
  }

  const goToDocument = (id: string) => {
    window.location.hash = `#/doc/${encodeURIComponent(id)}`
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
            {siteInfo?.version && (
              <span style={{ fontSize: '0.875rem', opacity: 0.7 }}>
                v{siteInfo.version}
              </span>
            )}
          </HeaderContent>
        </Header>
        <Main>
          {route.type === 'search' && (
            <SearchPage onDocumentClick={goToDocument} />
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
