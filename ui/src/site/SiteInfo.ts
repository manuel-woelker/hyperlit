export interface DocumentInfo {
  id: string,
  title: string;
}

export interface SiteInfo {
  title: string;
  documents: DocumentInfo[];
}