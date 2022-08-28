import * as proto from "vscode-languageserver-protocol";

const clientChannel = document.getElementById("channel-client") as HTMLTextAreaElement;
const serverChannel = document.getElementById("channel-server") as HTMLTextAreaElement;

export default class Tracer {
  static client(message: string): void {
    clientChannel.value += message;
    clientChannel.value += "\n";
  }

  static server(input: string | proto.Message): void {
    const message: string = typeof input === "string" ? input : JSON.stringify(input);
    serverChannel.value += message;
    serverChannel.value += "\n";
  }
}
