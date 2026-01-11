export interface BookStructure {
  title: string;
  chapters: ChapterStructure[];
}

export interface ChapterStructure {
  label: string;
  id: string,
  chapters: ChapterStructure[];
}

export interface DocumentInfo {
  id: string,
  title: string;
}

export interface SiteInfo {
  title: string;
  documents: DocumentInfo[];
}