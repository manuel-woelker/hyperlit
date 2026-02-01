export interface SiteInfo {
  title: string;
  description?: string;
  version?: string;
}

export interface DocumentSource {
  type: 'code_comment' | 'markdown_file';
  file_path: string;
  line_number: number;
  byte_range?: {
    start: number;
    end: number;
  };
}

export interface Document {
  id: string;
  title: string;
  content: string;
  source: DocumentSource;
  metadata?: Record<string, string>;
}

export interface SearchResult {
  document: Document;
  score: number;
  match_type: 'title' | 'content' | 'both';
}

export interface SearchResponse {
  query: string;
  results: SearchResult[];
}

const API_BASE = '/api';

export async function getSiteInfo(): Promise<SiteInfo> {
  const response = await fetch(`${API_BASE}/site`);
  if (!response.ok) {
    throw new Error(`Failed to fetch site info: ${response.statusText}`);
  }
  return response.json();
}

export async function getAllDocuments(): Promise<Document[]> {
  const response = await fetch(`${API_BASE}/documents`);
  if (!response.ok) {
    throw new Error(`Failed to fetch documents: ${response.statusText}`);
  }
  return response.json();
}

export async function searchDocuments(query: string): Promise<SearchResponse> {
  const response = await fetch(`${API_BASE}/search?q=${encodeURIComponent(query)}`);
  if (!response.ok) {
    throw new Error(`Failed to search documents: ${response.statusText}`);
  }
  return response.json();
}

export async function getDocument(id: string): Promise<Document> {
  const response = await fetch(`${API_BASE}/document/${encodeURIComponent(id)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch document: ${response.statusText}`);
  }
  return response.json();
}
