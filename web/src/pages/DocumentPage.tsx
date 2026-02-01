import { useState, useEffect } from 'react'
import styled from '@emotion/styled'
import ReactMarkdown from 'react-markdown'
import { getDocument, type Document } from '../api/client.ts'

const Container = styled.div`
  max-width: 900px;
  margin: 0 auto;
`

const BackButton = styled.button`
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid #ddd;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.875rem;
  color: #666;
  margin-bottom: 1.5rem;
  transition: all 0.2s;

  &:hover {
    background: #f5f5f5;
    border-color: #ccc;
  }
`

const DocumentHeader = styled.div`
  margin-bottom: 2rem;
  padding-bottom: 1.5rem;
  border-bottom: 2px solid #e0e0e0;
`

const DocumentTitle = styled.h1`
  margin: 0 0 1rem 0;
  font-size: 2rem;
  color: #1a1a2e;
  line-height: 1.3;
`

const SourceLink = styled.a`
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: #1565c0;
  text-decoration: none;
  font-family: monospace;
  background: #e3f2fd;
  padding: 0.375rem 0.75rem;
  border-radius: 4px;
  transition: background 0.2s;

  &:hover {
    background: #bbdefb;
  }
`

const ContentContainer = styled.div`
  background: white;
  border-radius: 8px;
  padding: 2rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);

  /* Markdown styles */
  h1, h2, h3, h4, h5, h6 {
    margin-top: 2rem;
    margin-bottom: 1rem;
    color: #1a1a2e;
  }

  h1 {
    font-size: 1.75rem;
    border-bottom: 2px solid #e0e0e0;
    padding-bottom: 0.5rem;
  }

  h2 {
    font-size: 1.5rem;
  }

  h3 {
    font-size: 1.25rem;
  }

  p {
    line-height: 1.7;
    margin-bottom: 1rem;
  }

  code {
    background: #f5f5f5;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 0.9em;
  }

  pre {
    background: #1a1a2e;
    color: #f8f8f2;
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
    margin: 1rem 0;

    code {
      background: transparent;
      padding: 0;
      color: inherit;
    }
  }

  ul, ol {
    margin-bottom: 1rem;
    padding-left: 1.5rem;
  }

  li {
    margin-bottom: 0.5rem;
    line-height: 1.6;
  }

  blockquote {
    border-left: 4px solid #1a1a2e;
    margin: 1rem 0;
    padding: 0.5rem 1rem;
    background: #f9f9f9;
    font-style: italic;
  }

  a {
    color: #1565c0;
    text-decoration: none;

    &:hover {
      text-decoration: underline;
    }
  }

  table {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
  }

  th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid #e0e0e0;
  }

  th {
    background: #f5f5f5;
    font-weight: 600;
  }

  hr {
    border: none;
    border-top: 2px solid #e0e0e0;
    margin: 2rem 0;
  }
`

const ErrorContainer = styled.div`
  text-align: center;
  padding: 3rem;
  color: #c62828;
`

const LoadingContainer = styled.div`
  text-align: center;
  padding: 3rem;
  color: #666;
`

interface DocumentPageProps {
  documentId: string
  onBack: () => void
}

export default function DocumentPage({ documentId, onBack }: DocumentPageProps) {
  const [document, setDocument] = useState<Document | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(true)
    setError(null)
    
    getDocument(documentId)
      .then(doc => {
        setDocument(doc)
        setLoading(false)
      })
      .catch(err => {
        setError(err.message)
        setLoading(false)
      })
  }, [documentId])

  if (loading) {
    return (
      <Container>
        <LoadingContainer>Loading document...</LoadingContainer>
      </Container>
    )
  }

  if (error || !document) {
    return (
      <Container>
        <BackButton onClick={onBack}>← Back to search</BackButton>
        <ErrorContainer>
          <p>Error loading document: {error || 'Document not found'}</p>
        </ErrorContainer>
      </Container>
    )
  }

  const sourceUrl = `vscode://file/${document.source.file_path}:${document.source.line_number}`

  return (
    <Container>
      <BackButton onClick={onBack}>← Back to search</BackButton>
      
      <DocumentHeader>
        <DocumentTitle>{document.title}</DocumentTitle>
        <SourceLink href={sourceUrl}>
          📄 {document.source.file_path}:{document.source.line_number}
        </SourceLink>
      </DocumentHeader>

      <ContentContainer>
        <ReactMarkdown>{document.content}</ReactMarkdown>
      </ContentContainer>
    </Container>
  )
}
