import { useState, useEffect, useCallback } from 'react'
import styled from '@emotion/styled'
import { searchDocuments, getAllDocuments, type Document, type SearchResult } from '../api/client.ts'
import { extractExcerpt, highlightMatches } from '../utils/searchHighlight.ts'

/* 📖 # Why light and airy search results?
Search results need to feel approachable and scannable. The design uses:

- **Soft cards**: White backgrounds with subtle borders create breathable space
- **Pastel accents**: Muted blues, purples, and greens for match type badges
- **Gentle interactions**: Soft shadows on hover provide feedback without jarring motion
- **Air gaps**: Generous padding and spacing prevent visual crowding

This creates a calm interface that helps users quickly scan through documentation
without feeling overwhelmed by dense visual information.
*/

const Container = styled.div`
  width: 100%;
  padding: 2rem;
`

const ResultsContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1rem;
`

const ResultCard = styled.div<{ color?: string; isSelected?: boolean }>`
  background: ${props => props.isSelected ? '#f0f9ff' : '#ffffff'};
  border-radius: 12px;
  padding: 1.5rem;
  border: 1px solid #e8eaed;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
  cursor: pointer;
  transition: all 0.2s ease;
  border-left: 3px solid ${props => props.isSelected ? '#3182ce' : (props.color || '#e2e8f0')};

  &:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
    transform: translateY(-1px);
    border-color: #d1d5db;
  }
`

const ResultTitle = styled.h3`
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  font-weight: 500;
  color: #2d3748;
  letter-spacing: -0.01em;
`

const ResultMeta = styled.div`
  display: flex;
  align-items: center;
  gap: 1rem;
  font-size: 0.875rem;
  color: #718096;
  margin-bottom: 0.75rem;
`

const MatchBadge = styled.span<{ type: string }>`
  padding: 0.375rem 0.75rem;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: lowercase;
  letter-spacing: 0.02em;
  background: ${props => {
    switch (props.type) {
      case 'title': return '#ebf8ff'
      case 'content': return '#faf5ff'
      case 'both': return '#f0fff4'
      default: return '#f7fafc'
    }
  }};
  color: ${props => {
    switch (props.type) {
      case 'title': return '#3182ce'
      case 'content': return '#805ad5'
      case 'both': return '#38a169'
      default: return '#718096'
    }
  }};
`

const ContentPreview = styled.p`
  margin: 0;
  color: #718096;
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
`

const HighlightedText = styled.mark`
  background-color: #fef08a;
  color: inherit;
  font-weight: 500;
  padding: 0 2px;
  border-radius: 2px;
`

const SourceInfo = styled.span`
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
  font-size: 0.8rem;
  background: #f7fafc;
  padding: 0.25rem 0.5rem;
  border-radius: 6px;
  color: #a0aec0;
  border: 1px solid #e8eaed;
`

const EmptyState = styled.div`
  text-align: center;
  padding: 3rem;
  color: #a0aec0;
`

const getMatchColor = (matchType: string) => {
  switch (matchType) {
    case 'title': return '#90cdf4'
    case 'content': return '#d6bcfa'
    case 'both': return '#9ae6b4'
    default: return '#e2e8f0'
  }
}

interface SearchPageProps {
  query: string
  onDocumentClick: (id: string) => void
  selectedDocId?: string | null
}

export default function SearchPage({ query, onDocumentClick, selectedDocId }: SearchPageProps) {
  const [results, setResults] = useState<SearchResult[]>([])
  const [allDocuments, setAllDocuments] = useState<Document[]>([])
  const [hasSearched, setHasSearched] = useState(false)

  // Load all documents on mount
  useEffect(() => {
    getAllDocuments()
      .then(docs => {
        setAllDocuments(docs)
        setResults(docs.map(doc => ({ document: doc, score: 0, match_type: 'both' as const })))
      })
      .catch(err => {
        console.error('Failed to load documents:', err)
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

    const timeoutId = setTimeout(() => {
      searchDocuments(query)
        .then(response => {
          setResults(response.results)
          setHasSearched(true)
        })
        .catch(err => {
          console.error('Search failed:', err)
        })
    }, 300)

    return () => {
      clearTimeout(timeoutId)
    }
  }, [query, allDocuments])

  const handleResultClick = useCallback((docId: string) => {
    onDocumentClick(docId)
  }, [onDocumentClick])

  return (
    <Container>
      {results.length === 0 && hasSearched && (
        <EmptyState>
          <p>No documents found matching "{query}"</p>
        </EmptyState>
      )}

      {results.length === 0 && !hasSearched && (
        <EmptyState>
          <p>No documents available</p>
        </EmptyState>
      )}

      <ResultsContainer>
        {results.map((result) => (
          <ResultCard
            key={result.document.id}
            color={getMatchColor(result.match_type)}
            isSelected={result.document.id === selectedDocId}
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
              {(() => {
                // For searches, show excerpt with highlighting for content/both matches
                if (hasSearched && query.trim() && (result.match_type === 'content' || result.match_type === 'both')) {
                  const excerpt = extractExcerpt(result.document.content, query)
                  if (excerpt) {
                    const segments = highlightMatches(excerpt.excerpt, query)
                    return (
                      <>
                        {segments.map((segment, i) =>
                          segment.isMatch ? (
                            <HighlightedText key={i}>{segment.text}</HighlightedText>
                          ) : (
                            <span key={i}>{segment.text.replace(/[#*`_]/g, '')}</span>
                          )
                        )}
                      </>
                    )
                  }
                }

                // Default preview: first 200 chars
                const cleanContent = result.document.content.slice(0, 200).replace(/[#*`_]/g, '')
                return cleanContent + (result.document.content.length > 200 ? '...' : '')
              })()}
            </ContentPreview>
          </ResultCard>
        ))}
      </ResultsContainer>
    </Container>
  )
}
