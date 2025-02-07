import * as vscode from "vscode";
import * as proto from "vscode-languageserver-protocol";
import { createConverter as createCodeConverter } from "vscode-languageclient/lib/common/codeConverter.js";
import { createConverter as createProtocolConverter } from "vscode-languageclient/lib/common/protocolConverter.js";

import getKeybindingsServiceOverride from "@codingame/monaco-vscode-keybindings-service-override";
import "@codingame/monaco-vscode-theme-defaults-default-extension";
import {
  RegisteredFileSystemProvider,
  registerFileSystemOverlay,
  RegisteredMemoryFile,
} from "@codingame/monaco-vscode-files-service-override";

import { WrapperConfig, MonacoEditorLanguageClientWrapper } from "monaco-editor-wrapper";

import Client from "./client";
import Server from "./server";
import { configureMonacoWorkers } from "./workers";
import { FromServer, IntoServer } from "./codec";
import Language from "./language";

import polarityLanguageConfig from "./language-configuration.json?raw";
import polarityTextmateGrammar from "./pol.tmLanguage.json?raw";

const code2Protocol = createCodeConverter();
const protocol2Code = createProtocolConverter(undefined, true, true);

export default class App {
  private diagnosticCollection: vscode.DiagnosticCollection | undefined;
  private client: Client | undefined;
  private currentDocument: vscode.TextDocument | undefined;
  private wrapper: MonacoEditorLanguageClientWrapper | undefined;
  private fileSystemProvider: RegisteredFileSystemProvider | undefined;
  private inMemoryFile: RegisteredMemoryFile | undefined;
  private inMemoryFileUri: vscode.Uri;
  private language: Language | undefined;

  constructor() {
    this.inMemoryFileUri = vscode.Uri.file("/examples/demo.pol");
  }

  async createEditor(client: Client): Promise<void> {
    this.client = client;

    const htmlContainer = document.getElementById("editor");
    if (!htmlContainer) {
      throw new Error("HTML container for Monaco editor not found.");
    }

    const languageId = "polarity";
    const extensionFilesOrContents = new Map<string, string | URL>();
    extensionFilesOrContents.set("/language-configuration.json", JSON.stringify(polarityLanguageConfig));
    extensionFilesOrContents.set("/syntaxes/pol.tmLanguage.json", JSON.stringify(polarityTextmateGrammar));

    const wrapperConfig: WrapperConfig = {
      $type: "extended",
      htmlContainer,
      logLevel: DEBUG ? vscode.LogLevel.Debug : vscode.LogLevel.Off,
      vscodeApiConfig: {
        serviceOverrides: {
          ...getKeybindingsServiceOverride(),
        },
        userConfiguration: {
          json: JSON.stringify({
            "workbench.colorTheme":
              window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches
                ? "Default Dark Modern"
                : "Default Light Modern",
            "editor.guides.bracketPairsHorizontal": "active",
            "editor.lightbulb.enabled": "On",
            "editor.experimental.asyncTokenization": true,
          }),
        },
      },
      extensions: [
        {
          config: {
            name: "polarity",
            publisher: "polarity-lang",
            version: "0.0.1",
            engines: {
              vscode: "*",
            },
            contributes: {
              languages: [
                {
                  id: "polarity",
                  extensions: [".pol"],
                  aliases: ["pol"],
                  configuration: "./language-configuration.json",
                },
              ],
              grammars: [
                {
                  language: "polarity",
                  scopeName: "source.pol",
                  path: "./syntaxes/pol.tmLanguage.json",
                },
              ],
            },
          },
          filesOrContents: extensionFilesOrContents,
        },
      ],
      editorAppConfig: {
        codeResources: {
          main: {
            text: "",
            uri: this.inMemoryFileUri.toString(),
            fileExt: "pol",
          },
        },
        monacoWorkerFactory: configureMonacoWorkers,
      },
    };

    this.wrapper = new MonacoEditorLanguageClientWrapper();
    await this.wrapper.init(wrapperConfig);
    await this.wrapper.start();

    this.diagnosticCollection = vscode.languages.createDiagnosticCollection("pol");
    this.fileSystemProvider = new RegisteredFileSystemProvider(false);
    this.inMemoryFile = new RegisteredMemoryFile(this.inMemoryFileUri, "");
    this.fileSystemProvider.registerFile(this.inMemoryFile);
    registerFileSystemOverlay(1, this.fileSystemProvider);

    Language.registerLanguageFeatures(client);

    vscode.workspace.onDidChangeTextDocument((e) => {
      if (!this.client || !this.currentDocument) return;
      if (e.document.uri.toString() !== this.currentDocument.uri.toString()) return;

      // For FULL sync, we need to send the entire updated text content
      const fullText = e.document.getText();

      this.client.notify(proto.DidChangeTextDocumentNotification.type.method, {
        textDocument: code2Protocol.asVersionedTextDocumentIdentifier(e.document),
        contentChanges: [
          {
            text: fullText,
          },
        ],
      } as proto.DidChangeTextDocumentParams);
    });

    client.pushAfterInitializeHook(async () => {
      if (!this.currentDocument) return;
      this.client?.notify(proto.DidOpenTextDocumentNotification.type.method, {
        textDocument: {
          uri: this.currentDocument.uri.toString(),
          languageId: languageId,
          version: this.currentDocument.version,
          text: this.currentDocument.getText(),
        },
      } as proto.DidOpenTextDocumentParams);

      await Promise.resolve();
    });

    client.addMethod(proto.PublishDiagnosticsNotification.type.method, async (params) => {
      if (!params) return;
      const p = params as proto.PublishDiagnosticsParams;
      const diagnostics = (await protocol2Code.asDiagnostics(p.diagnostics)) ?? [];
      this.diagnosticCollection?.set(vscode.Uri.parse(p.uri), diagnostics);
    });

    const handleHashChange = async () => {
      const filepath = location.hash.slice(1).trim();
      if (filepath === "") {
        await this.updateInMemoryFileContent("");
        return;
      }

      const url = `${location.protocol}//${location.host}/examples/${encodeURIComponent(filepath)}`;
      const response = await fetch(url);
      const text = await response.text();

      await this.updateInMemoryFileContent(text);
    };

    window.addEventListener("hashchange", () => {
      void handleHashChange().catch((error) => {
        console.error("Error handling hash change:", error);
      });
    });

    await handleHashChange();
  }

  async run(): Promise<void> {
    const fromServer = FromServer.create();
    const intoServer = new IntoServer();
    const client = new Client(fromServer, intoServer);
    const server = await Server.initialize(intoServer, fromServer);
    await this.createEditor(client);
    await Promise.all([server.start(), client.start()]);
  }

  private async updateInMemoryFileContent(newContent: string): Promise<void> {
    const encoder = new TextEncoder();
    const encodedContent = encoder.encode(newContent);
    await this.inMemoryFile?.write(encodedContent);

    if (this.currentDocument && this.currentDocument.uri.toString() === this.inMemoryFileUri.toString()) {
      const edit: vscode.WorkspaceEdit = new vscode.WorkspaceEdit();
      const fullRange = new vscode.Range(
        this.currentDocument.positionAt(0),
        this.currentDocument.positionAt(this.currentDocument.getText().length),
      );
      edit.replace(this.currentDocument.uri, fullRange, newContent);
      await vscode.workspace.applyEdit(edit);
      this.client?.notify(proto.DidChangeTextDocumentNotification.type.method, {
        textDocument: code2Protocol.asVersionedTextDocumentIdentifier(this.currentDocument),
        contentChanges: [
          {
            text: newContent,
          },
        ],
      } as proto.DidChangeTextDocumentParams);
    } else {
      // Open the in-memory document if it's not already open
      const doc = await vscode.workspace.openTextDocument(this.inMemoryFileUri);
      this.currentDocument = doc;
      // `preserveFocus: true` is necessary to prevent the editor from grapping focus on page load.
      // This resulted the landing page to jump to the editor immediately after page load.
      await vscode.window.showTextDocument(doc, { preview: false, preserveFocus: true });
    }
  }
}
