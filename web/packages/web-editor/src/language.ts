import * as vscode from "vscode";
import * as proto from "vscode-languageserver-protocol";
import { createConverter as createCodeConverter } from "vscode-languageclient/lib/common/codeConverter.js";
import { createConverter as createProtocolConverter } from "vscode-languageclient/lib/common/protocolConverter.js";
import Client from "polarity-lang-lsp-web/client";

const code2Protocol = createCodeConverter();
const protocol2Code = createProtocolConverter(undefined, true, true);

export default class Language {
  static id: string = "polarity";

  static registerLanguageFeatures(client: Client): void {
    vscode.languages.registerHoverProvider(this.id, {
      async provideHover(document, position, token) {
        void token;
        const response = (await client.request(proto.HoverRequest.type.method, {
          textDocument: code2Protocol.asTextDocumentIdentifier(document),
          position: code2Protocol.asPosition(position),
        })) as proto.Hover | null;

        if (!response) {
          return undefined;
        }
        return protocol2Code.asHover(response);
      },
    });

    vscode.languages.registerCodeActionsProvider(this.id, {
      async provideCodeActions(document, range, context, token) {
        void token;
        const pContext = await code2Protocol.asCodeActionContext(context);
        pContext.diagnostics = pContext.diagnostics ?? [];

        const response = (await client.request(proto.CodeActionRequest.type.method, {
          textDocument: code2Protocol.asTextDocumentIdentifier(document),
          range: code2Protocol.asRange(range),
          context: pContext,
        })) as proto.CodeAction[] | null;

        if (!response) {
          return [];
        }

        const actions = await Promise.all(response.map((action) => protocol2Code.asCodeAction(action)));
        return actions.filter((a): a is vscode.CodeAction => a !== undefined);
      },
    });

    vscode.languages.registerDocumentFormattingEditProvider(this.id, {
      async provideDocumentFormattingEdits(document, options, token) {
        void token;
        const response = (await client.request(proto.DocumentFormattingRequest.type.method, {
          textDocument: code2Protocol.asTextDocumentIdentifier(document),
          options: {
            tabSize: options.tabSize,
            insertSpaces: options.insertSpaces,
          },
        })) as proto.TextEdit[] | null;

        if (response) {
          const textEdits = await protocol2Code.asTextEdits(response);
          return textEdits || [];
        }
        return [];
      },
    });
  }
}
