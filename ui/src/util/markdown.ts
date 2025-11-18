import {micromark} from "micromark";
import {frontmatter} from "micromark-extension-frontmatter";
import {gfmTable, gfmTableHtml} from "micromark-extension-gfm-table";


export function markdownToHtml(markdown: string): string {
  console.time("Transform markdown to html");
  let html = micromark(markdown, {
    extensions: [gfmTable(), frontmatter(["yaml", "toml"])],
    htmlExtensions: [gfmTableHtml()]
//        htmlExtensions: [frontmatterHtml()]
  });
  console.timeEnd("Transform markdown to html");
  return html;
}
