import DOMPurify from "dompurify";

const ALLOWED_TAGS = [
  "a", "b", "blockquote", "br", "code", "div", "em", "h1", "h2", "h3",
  "h4", "h5", "h6", "hr", "i", "img", "li", "ol", "p", "pre", "span",
  "strong", "table", "tbody", "td", "th", "thead", "tr", "u", "ul",
];

const ALLOWED_ATTR = [
  "href", "src", "alt", "title", "style", "class", "width", "height",
  "align", "valign", "colspan", "rowspan", "border", "cellpadding",
  "cellspacing", "bgcolor", "color", "target",
];

export function sanitizeHtml(dirty: string): string {
  return DOMPurify.sanitize(dirty, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ALLOW_DATA_ATTR: false,
    ADD_ATTR: ["target"],
  });
}
