import debounce from "debounce";
import * as monaco from "monaco-editor-core";
import { MonacoToProtocolConverter, PublishDiagnosticsParams } from "monaco-languageclient";
import * as proto from "vscode-languageserver-protocol";

import Client from "./client";
import { FromServer, IntoServer } from "./codec";
import Language, { protocolToMonaco } from "./language";
import Server from "./server";

class Environment implements monaco.Environment {
  getWorkerUrl(moduleId: string, label: string) {
    if (label === "editorWorkerService") {
      const path = parentPath(location.pathname);
      return `${path}editor.worker.bundle.js`;
    }
    throw new Error(`getWorkerUrl: unexpected ${JSON.stringify({ moduleId, label })}`);
  }
}

const monacoToProtocol = new MonacoToProtocolConverter(monaco);

export default class App {
  readonly #window: Window & monaco.Window & typeof globalThis = self;

  readonly #intoServer: IntoServer = new IntoServer();
  readonly #fromServer: FromServer = FromServer.create();

  initializeMonaco(): void {
    this.#window.MonacoEnvironment = new Environment();
  }

  createModel(client: Client): monaco.editor.ITextModel {
    const language = Language.initialize(client);

    const id = language.id;
    const uri = monaco.Uri.parse("inmemory:///examples/demo.pol");

    const model = monaco.editor.createModel("", id, uri);

    model.onDidChangeContent(
      debounce(() => {
        const text = model.getValue();
        client.notify(proto.DidChangeTextDocumentNotification.type.method, {
          textDocument: {
            version: 0,
            uri: model.uri.toString(),
          },
          contentChanges: [
            {
              range: monacoToProtocol.asRange(model.getFullModelRange()),
              text,
            },
          ],
        } as proto.DidChangeTextDocumentParams);
      }, 200),
    );

    // eslint-disable-next-line @typescript-eslint/require-await
    client.pushAfterInitializeHook(async () => {
      client.notify(proto.DidOpenTextDocumentNotification.type.method, {
        textDocument: {
          uri: model.uri.toString(),
          languageId: language.id,
          version: 0,
          text: model.getValue(),
        },
      } as proto.DidOpenTextDocumentParams);
    });

    client.addMethod(proto.PublishDiagnosticsNotification.type.method, (params) => {
      const { diagnostics } = params as PublishDiagnosticsParams;

      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const markers: monaco.editor.IMarkerData[] = protocolToMonaco.asDiagnostics(diagnostics);

      monaco.editor.setModelMarkers(model, language.id, markers);
    });

    async function handleHashChange() {
      const filepath = location.hash.slice(1);
      if (filepath === "") {
        model.setValue("");
        return;
      }
      const path = parentPath(location.pathname);
      const url = `${location.protocol}//${location.host}${path}examples/${filepath}`;
      const response = await fetch(url);
      if (!response.ok) {
        model.setValue("");
      } else {
        const text = await response.text();
        model.setValue(text);
      }
    }

    addEventListener("hashchange", (event) => {
      void event;
      void handleHashChange();
    });

    return model;
  }

  createEditor(client: Client): void {
    const container = document.getElementById("editor")!; // eslint-disable-line @typescript-eslint/no-non-null-assertion
    this.initializeMonaco();
    const model = this.createModel(client);
    monaco.editor.create(container, {
      model,
      automaticLayout: true,
      scrollBeyondLastLine: false,
      theme: window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches ? "vs-dark" : "vs-light",
      scrollbar: {
        // Pass scroll events to parent
        alwaysConsumeMouseWheel: false,
      },
    });

    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        if (mutation.type === "attributes") {
          const thm = document.documentElement.getAttribute("data-theme");
          if (thm == "dark") {
            monaco.editor.setTheme("vs-dark");
          } else if (thm == "light") {
            monaco.editor.setTheme("vs-light");
          }
        }
      });
    });
    observer.observe(document.documentElement, { attributes: true });
  }

  async run(): Promise<void> {
    const client = new Client(this.#fromServer, this.#intoServer);
    const server = await Server.initialize(this.#intoServer, this.#fromServer);
    this.createEditor(client);
    window.dispatchEvent(new HashChangeEvent("hashchange"));
    await Promise.all([server.start(), client.start()]);
  }
}

function parentPath(pathname: string): string {
  if (pathname.endsWith("/")) {
    pathname = pathname.slice(0, -1);
  }
  if (pathname === "") {
    return "/";
  }
  return pathname.slice(0, pathname.lastIndexOf("/") + 1);
}
