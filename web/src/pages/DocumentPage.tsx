import { useState, useEffect } from 'react'
import styled from '@emotion/styled'
import ReactMarkdown from 'react-markdown'
import { getDocument, type Document } from '../api/client.ts'

/* 📖 # Why light colors for document reading?
Reading technical documentation requires focus and extended attention. The design uses:

- **Clean white canvas**: Documents appear on pure white (#ffffff) for maximum readability
- **Soft code blocks**: Light gray (#f7fafc) backgrounds for inline code reduce visual noise
- **Gentle accent borders**: Muted blues for links and source references guide without distraction
- **Breathing room**: Generous padding and line height reduce eye strain during long reading

The goal is to create an environment where the documentation content itself is the star,
with the interface fading into the background.
*/

const Container = styled.div`
  max-width: 900px;
  margin: 0 auto;
`

const BackButton = styled.button`
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.875rem;
  color: #718096;
  margin-bottom: 1.5rem;
  transition: all 0.2s ease;

  &:hover {
    background: #f7fafc;
    border-color: #cbd5e0;
    color: #4a5568;
  }
`

const DocumentHeader = styled.div`
  margin-bottom: 2.5rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid #e8eaed;
`

const DocumentTitle = styled.h1`
  margin: 0 0 1rem 0;
  font-size: 2rem;
  font-weight: 600;
  color: #2d3748;
  line-height: 1.3;
  letter-spacing: -0.02em;
`

const SourceLink = styled.a`
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: #3182ce;
  text-decoration: none;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
  background: #ebf8ff;
  padding: 0.5rem 0.875rem;
  border-radius: 8px;
  transition: all 0.2s ease;
  border: 1px solid #bee3f8;

  &:hover {
    background: #bce3ff;
    border-color: #90cdf4;
  }
`

const ContentContainer = styled.div`
  background: #ffffff;
  border-radius: 16px;
  padding: 2.5rem;
  border: 1px solid #e8eaed;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);

  /* Markdown styles - light and airy */
  h1, h2, h3, h4, h5, h6 {
    margin-top: 2.5rem;
    margin-bottom: 1rem;
    color: #2d3748;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  h1 {
    font-size: 1.75rem;
    border-bottom: 1px solid #e8eaed;
    padding-bottom: 0.75rem;
    margin-top: 0;
  }

  h2 {
    font-size: 1.5rem;
  }

  h3 {
    font-size: 1.25rem;
  }

  p {
    line-height: 1.8;
    margin-bottom: 1.25rem;
    color: #4a5568;
  }

  code {
    background: #f7fafc;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 0.875em;
    color: #2d3748;
    border: 1px solid #e8eaed;
  }

  pre {
    background: #f7fafc;
    color: #2d3748;
    padding: 1.25rem;
    border-radius: 12px;
    overflow-x: auto;
    margin: 1.5rem 0;
    border: 1px solid #e8eaed;

    code {
      background: transparent;
      padding: 0;
      border: none;
      color: inherit;
    }
  }

  ul, ol {
    margin-bottom: 1.25rem;
    padding-left: 1.5rem;
    color: #4a5568;
  }

  li {
    margin-bottom: 0.5rem;
    line-height: 1.7;
  }

  blockquote {
    border-left: 3px solid #bee3f8;
    margin: 1.5rem 0;
    padding: 1rem 1.5rem;
    background: #f7fafc;
    border-radius: 0 8px 8px 0;
    color: #4a5568;
  }

  a {
    color: #3182ce;
    text-decoration: none;
    transition: color 0.2s;

    &:hover {
      color: #2c5282;
      text-decoration: underline;
    }
  }

  table {
    width: 100%;
    border-collapse: collapse;
    margin: 1.5rem 0;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid #e8eaed;
  }

  th, td {
    padding: 0.875rem;
    text-align: left;
    border-bottom: 1px solid #e8eaed;
  }

  th {
    background: #f7fafc;
    font-weight: 600;
    color: #2d3748;
  }

  td {
    color: #4a5568;
  }

  hr {
    border: none;
    border-top: 1px solid #e8eaed;
    margin: 2.5rem 0;
  }
`

const ErrorContainer = styled.div`
  text-align: center;
  padding: 3rem;
  color: #e53e3e;
  background: #fff5f5;
  border-radius: 12px;
  border: 1px solid #fed7d7;
`

const LoadingContainer = styled.div`
  text-align: center;
  padding: 3rem;
  color: #a0aec0;
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
