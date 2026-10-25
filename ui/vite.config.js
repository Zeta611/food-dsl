import { resolve } from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [
    {
      name: "elm-live-reload",
      configureServer(server) {
        const elmOutput = resolve("public/elm.js");
        server.watcher.add(elmOutput);
        server.watcher.on("change", (file) => {
          if (resolve(file) === elmOutput) {
            server.ws.send({ type: "full-reload" });
          }
        });
      },
    },
  ],
});
