import styled from '@emotion/styled'

const Container = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 3rem;
  color: #a0aec0;
  text-align: center;
`

const Message = styled.p`
  font-size: 1rem;
  margin: 0;
  line-height: 1.6;
`

export default function EmptyDocumentPlaceholder() {
  return (
    <Container>
      <Message>Select a document from the search results to view</Message>
    </Container>
  )
}
