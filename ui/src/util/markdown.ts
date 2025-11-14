import {micromark} from "micromark";
import {frontmatter} from "micromark-extension-frontmatter";


export function markdownToHtml(markdown: string): string {
  console.time("Transform markdown to html");
  let html = micromark(markdown, {
    extensions: [frontmatter(["yaml", "toml"])],
//        htmlExtensions: [frontmatterHtml()]
  });
  console.timeEnd("Transform markdown to html");
  return html;
}
