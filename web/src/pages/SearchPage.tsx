import { useState, useEffect, useCallback } from 'react'
import styled from '@emotion/styled'
import { searchDocuments, getAllDocuments, type Document, type SearchResult } from '../api/client.ts'

const Container = styled.div`
  max-width: 800px;
  margin: 0 auto;
`

const SearchInput = styled.input`
  width: 100%;
  padding: 1rem 1.5rem;
  font-size: 1.125rem;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  outline: none;
  transition: border-color 0.2s;
  margin-bottom: 2rem;

  &:focus {
    border-color: #1a1a2e;
  }

  &::placeholder {
    color: #999;
  }
`

const ResultsContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1rem;
`

const ResultCard = styled.div`
  background: white;
  border-radius: 8px;
  padding: 1.5rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  cursor: pointer;
  transition: box-shadow 0.2s, transform 0.2s;
  border-left: 4px solid ${props => props.color || '#1a1a2e'};

  &:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    transform: translateY(-2px);
  }
`

const ResultTitle = styled.h3`
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  color: #1a1a2e;
`

const ResultMeta = styled.div`
  display: flex;
  align-items: center;
  gap: 1rem;
  font-size: 0.875rem;
  color: #666;
  margin-bottom: 0.75rem;
`

const MatchBadge = styled.span<{ type: string }>`
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  background: ${props => {
    switch (props.type) {
      case 'title': return '#e3f2fd'
      case 'content': return '#f3e5f5'
      case 'both': return '#e8f5e9'
      default: return '#f5f5f5'
    }
  }};
  color: ${props => {
    switch (props.type) {
      case 'title': return '#1565c0'
      case 'content': return '#7b1fa2'
      case 'both': return '#2e7d32'
      default: return '#616161'
    }
  }};
`

const ContentPreview = styled.p`
  margin: 0;
  color: #555;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
`

const SourceInfo = styled.span`
  font-family: monospace;
  font-size: 0.8rem;
  background: #f5f5f5;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
`

const EmptyState = styled.div`
  text-align: center;
  padding: 3rem;
  color: #666;
`

const LoadingState = styled.div`
  text-align: center;
  padding: 2rem;
  color: #666;
`

const getMatchColor = (matchType: string) => {
  switch (matchType) {
    case 'title': return '#1565c0'
    case 'content': return '#7b1fa2'
    case 'both': return '#2e7d32'
    default: return '#1a1a2e'
  }
}

interface SearchPageProps {
  onDocumentClick: (id: string) => void
}

export default function SearchPage({ onDocumentClick }: SearchPageProps) {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [allDocuments, setAllDocuments] = useState<Document[]>([])
  const [loading, setLoading] = useState(false)
  const [hasSearched, setHasSearched] = useState(false)

  // Load all documents on mount
  useEffect(() => {
    setLoading(true)
    getAllDocuments()
      .then(docs => {
        setAllDocuments(docs)
        setResults(docs.map(doc => ({ document: doc, score: 0, match_type: 'both' as const })))
        setLoading(false)
      })
      .catch(err => {
        console.error('Failed to load documents:', err)
        setLoading(false)
      })
  }, [])

  // Debounced search
  useEffect(() => {
    if (!query.trim()) {
      // Show all documents when query is empty
      setResults(allDocuments.map(doc => ({ document: doc, score: 0, match_type: 'both' as const })))
      setHasSearched(false)
      return
    }

    setLoading(true)
    const timeoutId = setTimeout(() => {
      searchDocuments(query)
        .then(response => {
          setResults(response.results)
          setHasSearched(true)
          setLoading(false)
        })
        .catch(err => {
          console.error('Search failed:', err)
          setLoading(false)
        })
    }, 300)

    return () => clearTimeout(timeoutId)
  }, [query, allDocuments])

  const handleResultClick = useCallback((docId: string) => {
    onDocumentClick(docId)
  }, [onDocumentClick])

  return (
    <Container>
      <SearchInput
        type="text"
        placeholder="Search documentation..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        autoFocus
      />

      {loading && (
        <LoadingState>Loading...</LoadingState>
      )}

      {!loading && results.length === 0 && hasSearched && (
        <EmptyState>
          <p>No documents found matching "{query}"</p>
        </EmptyState>
      )}

      {!loading && results.length === 0 && !hasSearched && (
        <EmptyState>
          <p>No documents available</p>
        </EmptyState>
      )}

      <ResultsContainer>
        {results.map((result) => (
          <ResultCard
            key={result.document.id}
            color={getMatchColor(result.match_type)}
            onClick={() => handleResultClick(result.document.id)}
          >
            <ResultTitle>{result.document.title}</ResultTitle>
            <ResultMeta>
              {hasSearched && (
                <MatchBadge type={result.match_type}>
                  {result.match_type} match
                </MatchBadge>
              )}
              <SourceInfo>
                {result.document.source.file_path}:{result.document.source.line_number}
              </SourceInfo>
            </ResultMeta>
            <ContentPreview>
              {result.document.content.slice(0, 200).replace(/[#*`_]/g, '')}
              {result.document.content.length > 200 ? '...' : ''}
            </ContentPreview>
          </ResultCard>
        ))}
      </ResultsContainer>
    </Container>
  )
}
