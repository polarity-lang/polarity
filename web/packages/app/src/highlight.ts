import { Language } from "highlight.js";
import hljs from "highlight.js";
import "../assets/highlight.scss";

const COMMENT = hljs.COMMENT("--", "$");
const PUNCTUATION = {
  match: /=>|,|:|\.|{|}|\(|\)/,
  className: "punctuation",
};
const UPPER_IDENT = {
  match: /[A-Z][a-zA-Z0-9_]*[']*/,
  className: "title.class",
  relevance: 1,
};
const xfn: Language = {
  case_insensitive: false,
  keywords: {
    keyword: ["data", "codata", "impl", "def", "codef", "match", "comatch", "absurd"],
    built_in: ["Type"],
  },
  contains: [COMMENT, PUNCTUATION, UPPER_IDENT],
};

export const register = () => {
  document.addEventListener("DOMContentLoaded", () => {
    hljs.registerLanguage("xfn", () => xfn);
    hljs.highlightAll();
  });
};
