import "./render";

import init, { greet } from "../../binding/pkg";

await init();
greet();

Elm.Main.init({
  node: document.querySelector("main"),
});
