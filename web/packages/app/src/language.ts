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
    const id = "xfunc";
    const aliases: string[] = [];
    const extensions = [".xfn"];
    const mimetypes: string[] = [];
    return { id, extensions, aliases, mimetypes };
  }

  private registerLanguage(client: Client): void {
    void client;
    monaco.languages.register(Language.extensionPoint());
    monaco.languages.registerDocumentSymbolProvider(this.id, {
      // eslint-disable-next-line
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
