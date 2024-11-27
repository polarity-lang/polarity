// @ts-check

const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WebpackWatchFilesPlugin = require("webpack-watch-files-plugin").default;
const path = require("path");
const webpack = require("webpack");
const Handlebars = require("handlebars");
const { marked } = require("marked");
const fs = require("fs");

const loadExamples = () => {
  const dir = fs.readdirSync("../../../examples");
  const files = dir.filter((filename) => filename.endsWith(".pol")).map((filename) => `../../../examples/${filename}`);
  let out = [];
  for (const filename of files) {
    out.push(path.basename(filename, ".pol"));
  }
  return out;
};

module.exports = (env, argv) => {
  const prod = argv.mode === "production";
  console.log(prod ? "Production mode" : "Development mode");

  /** @type {import("webpack").Configuration & { devServer?: import("webpack-dev-server").Configuration } } */
  const config = {
    experiments: {
      asyncWebAssembly: true,
    },
    mode: "production",
    target: "web",
    entry: {
      editor: "./src/editor/index.ts",
      "editor.worker": "monaco-editor-core/esm/vs/editor/editor.worker.js",
    },
    resolve: {
      alias: {
        vscode: require.resolve("monaco-languageclient/vscode-compatibility"),
      },
      extensions: [".ts", ".js", ".json", ".ttf"],
      fallback: {
        fs: false,
        child_process: false,
        net: false,
        crypto: false,
        path: require.resolve("path-browserify"),
      },
    },
    output: {
      globalObject: "self",
      filename: "[name].bundle.js",
      path: path.resolve(__dirname, "dist"),
    },
    module: {
      rules: [
        {
          test: /\.ts?$/,
          loader: "esbuild-loader",
          options: {
            loader: "ts",
            target: "es2022",
            minify: true,
          },
        },
        {
          test: /\.css$/,
          use: ["style-loader", "css-loader"],
        },
        {
          test: /\.s[ac]ss$/i,
          use: ["style-loader", "css-loader", "sass-loader"],
        },
        {
          test: /\.(woff|woff2|eot|ttf|otf)$/i,
          type: "asset/resource",
        },
        {
          test: /\.html$/,
          loader: "html-loader",
          options: {
            preprocessor: (content, loaderContext) => {
              let result;

              const input = { examples: loadExamples() };

              try {
                result = Handlebars.compile(content)(input);
              } catch (error) {
                loaderContext.emitError(error);

                return content;
              }

              return result;
            },
          },
        },
      ],
    },
    plugins: [
      new webpack.ProgressPlugin(),
      new CleanWebpackPlugin(),
      new CopyWebpackPlugin({
        patterns: [
          { from: "../../../examples", to: "examples" },
          { from: "../../../std", to: "std" }
        ],
      }),
      new webpack.DefinePlugin({
        DEBUG: !prod,
      }),
      new HtmlWebpackPlugin({
        template: "assets/editor.html",
        filename: "editor/index.html",
        chunks: ["editor", "editor.worker"],
        scriptLoading: "defer",
      }),
    ],
    optimization: {
      minimize: prod,
      runtimeChunk: "single",
    },
    performance: {
      hints: false,
    },
    devServer: {
      static: {
        directory: path.join(__dirname, "dist"),
      },
      port: 9000,
      client: {
        progress: true,
        reconnect: false,
      },
      devMiddleware: {
        writeToDisk: true,
      },
    },
  };
  return config;
};
