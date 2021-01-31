const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
  webpack: {
    configure: function override(config, dev) {
      /**
       * Add WASM support. SO: https://stackoverflow.com/questions/59319775
       */

      // Make file-loader ignore WASM files
      const wasmExtensionRegExp = /\.wasm$/;
      config.resolve.extensions.push('.wasm');
      config.module.rules.forEach(rule => {
        (rule.oneOf || []).forEach(oneOf => {
          if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
            oneOf.exclude.push(wasmExtensionRegExp);
          }
        });
      });

      // Add a dedicated loader for WASM
      config.module.rules.push({
        test: wasmExtensionRegExp,
        include: path.resolve(__dirname, 'src'),
        use: [{ loader: require.resolve('wasm-loader'), options: {} }]
      });

      config.plugins.push(new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "backend"),
      }));

      return config;
    }
  }
}
