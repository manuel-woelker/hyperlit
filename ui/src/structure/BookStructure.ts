export interface BookStructure {
  title: string;
  chapters: ChapterStructure[];
}

export interface ChapterStructure {
  label: string;
  id: string,
  chapters: ChapterStructure[];
}