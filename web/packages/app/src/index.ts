import "../assets/index.module.css";

import App from "./app";
import * as highlight from "./highlight";

highlight.register();

const app = new App();
app.run().catch(console.error);
