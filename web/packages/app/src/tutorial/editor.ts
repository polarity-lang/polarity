const editorIFrame = <HTMLIFrameElement>document.getElementById("editor-frame");

export const register = () => {
  editorIFrame.onload = function () {
    addEventListener("hashchange", (event) => {
      void event;
      // eslint-disable-next-line  @typescript-eslint/no-non-null-assertion
      editorIFrame.contentWindow!.location.hash = location.hash;
    });

    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.type === "attributes") {
          const thm = document.documentElement.getAttribute("data-theme");
          // eslint-disable-next-line  @typescript-eslint/no-non-null-assertion
          editorIFrame.contentDocument!.documentElement.setAttribute("data-theme", thm!);
        }
      });
    });
    observer.observe(document.documentElement, { attributes: true });
  };
};
