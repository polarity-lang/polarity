// import * as jsrpc from "json-rpc-2.0";
import { MonacoToProtocolConverter, ProtocolToMonacoConverter } from "monaco-languageclient";
import * as monaco from "monaco-editor-core";
import * as proto from "vscode-languageserver-protocol";

import Client from "./client";

export const monacoToProtocol = new MonacoToProtocolConverter(monaco);
export const protocolToMonaco = new ProtocolToMonacoConverter(monaco);

let language: null | Language;

export default class Language implements monaco.languages.ILanguageExtensionPoint {
  readonly id: string;
  readonly aliases: string[];
  readonly extensions: string[];
  readonly mimetypes: string[];

  private constructor(client: Client) {
    const { id, aliases, extensions, mimetypes } = Language.extensionPoint();
    this.id = id;
    this.aliases = aliases;
    this.extensions = extensions;
    this.mimetypes = mimetypes;
    this.registerLanguage(client);
  }

  static extensionPoint(): monaco.languages.ILanguageExtensionPoint & {
    aliases: string[];
    extensions: string[];
    mimetypes: string[];
  } {
    const id = "xfn";
    const aliases: string[] = [];
    const extensions = [".xfn"];
    const mimetypes: string[] = [];
    return { id, extensions, aliases, mimetypes };
  }

  private registerLanguage(client: Client): void {
    void client;
    monaco.languages.register(Language.extensionPoint());
    monaco.languages.setMonarchTokensProvider(this.id, Language.syntaxDefinition());
    monaco.languages.registerDocumentSymbolProvider(this.id, {
      async provideDocumentSymbols(model, token): Promise<monaco.languages.DocumentSymbol[]> {
        void token;
        const response = await (client.request(proto.DocumentSymbolRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
        } as proto.DocumentSymbolParams) as Promise<proto.SymbolInformation[]>);

        const uri = model.uri.toString();

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.DocumentSymbol[] = protocolToMonaco.asSymbolInformations(response, uri);

        return result;
      },
    });
    monaco.languages.registerHoverProvider(this.id, {
      async provideHover(model, position, token): Promise<monaco.languages.Hover> {
        void token;
        const response = await (client.request(proto.HoverRequest.type.method, {
          textDocument: {
            version: 0,
            uri: model.uri.toString(),
          },
          position: monacoToProtocol.asPosition(position.lineNumber, position.column),
        } as proto.TextDocumentPositionParams) as Promise<proto.Hover>);

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.Hover = protocolToMonaco.asHover(response);

        return result;
      },
    });
    monaco.languages.registerCodeActionProvider(this.id, {
      async provideCodeActions(model, range, context, token): Promise<monaco.languages.CodeActionList> {
        void token;
        const response = await (client.request(proto.CodeActionRequest.type.method, {
          textDocument: {
            version: 0,
            uri: model.uri.toString(),
          },
          range: monacoToProtocol.asRange(range),
          context: monacoToProtocol.asCodeActionContext(context, []),
        } as proto.CodeActionParams) as Promise<proto.CodeAction[]>);

        if (response === null) {
          return { actions: [], dispose: () => undefined };
        }

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.CodeActionList = protocolToMonaco.asCodeActionList(response);

        console.log(result);
        console.log(model.uri);

        return result;
      },
    });
  }

  private static syntaxDefinition(): monaco.languages.IMonarchLanguage {
    return {
      keywords: ["data", "codata", "impl", "def", "codef", "match", "comatch", "absurd"],

      typeKeywords: ["Type"],

      operators: [";", "=>", ",", ":", "."],

      tokenizer: {
        root: [
          // identifiers and keywords
          [
            /[a-z_][a-zA-Z0-9_]*[']*/,
            { cases: { "@typeKeywords": "keyword", "@keywords": "keyword", "@default": "identifier" } },
          ],
          [/[A-Z][a-zA-Z0-9_]*[']*/, "type.identifier"],

          // whitespace
          { include: "@whitespace" },

          // delimiter
          [/[;,.]/, "delimiter"],
        ],

        comment: [[/--/, "comment"]],

        whitespace: [
          [/[ \t\r\n]+/, "white"],
          [/--.*$/, "comment"],
        ],
      },
    };
  }

  static initialize(client: Client): Language {
    if (null == language) {
      language = new Language(client);
    } else {
      console.warn("Language already initialized; ignoring");
    }
    return language;
  }
}
