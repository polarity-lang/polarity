import * as jsrpc from "json-rpc-2.0";
import * as proto from "vscode-languageserver-protocol";

import { Codec, FromServer, IntoServer } from "./codec";

const consoleChannel = document.getElementById("channel-console") as HTMLTextAreaElement;

export default class Client extends jsrpc.JSONRPCServerAndClient {
  afterInitializedHooks: (() => Promise<void>)[] = [];
  #fromServer: FromServer;

  constructor(fromServer: FromServer, intoServer: IntoServer) {
    super(
      new jsrpc.JSONRPCServer(),
      new jsrpc.JSONRPCClient(async (json: jsrpc.JSONRPCRequest) => {
        const encoded = Codec.encode(json);
        intoServer.enqueue(encoded);
        if (null != json.id) {
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          const response = await fromServer.responses.get(json.id)!;
          this.client.receive(response as jsrpc.JSONRPCResponse);
        }
      }),
    );
    this.#fromServer = fromServer;
  }

  async start(): Promise<void> {
    // process "window/logMessage": client <- server
    this.addMethod(proto.LogMessageNotification.type.method, (params) => {
      const { type, message } = params as { type: proto.MessageType; message: string };
      switch (type) {
        case proto.MessageType.Error: {
          consoleChannel.value += "[error] ";
          break;
        }
        case proto.MessageType.Warning: {
          consoleChannel.value += " [warn] ";
          break;
        }
        case proto.MessageType.Info: {
          consoleChannel.value += " [info] ";
          break;
        }
        case proto.MessageType.Log: {
          consoleChannel.value += "  [log] ";
          break;
        }
      }
      consoleChannel.value += message;
      consoleChannel.value += "\n";
      return;
    });

    // request "initialize": client <-> server
    await (this.request(proto.InitializeRequest.type.method, {
      processId: null,
      clientInfo: {
        name: "demo-language-client",
      },
      capabilities: {},
      rootUri: null,
    } as proto.InitializeParams) as Promise<jsrpc.JSONRPCResponse>);

    // notify "initialized": client --> server
    this.notify(proto.InitializedNotification.type.method, {});

    await Promise.all(this.afterInitializedHooks.map((f: () => Promise<void>) => f()));
    await Promise.all([this.processNotifications(), this.processRequests()]);
  }

  async processNotifications(): Promise<void> {
    for await (const notification of this.#fromServer.notifications) {
      await this.receiveAndSend(notification);
    }
  }

  async processRequests(): Promise<void> {
    for await (const request of this.#fromServer.requests) {
      await this.receiveAndSend(request);
    }
  }

  pushAfterInitializeHook(...hooks: (() => Promise<void>)[]): void {
    this.afterInitializedHooks.push(...hooks);
  }
}
