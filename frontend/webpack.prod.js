const config = require('./webpack.config');

const ClosureCompilerPlugin = require('webpack-closure-compiler');

module.exports = {
  ...config,
  mode: 'production',
  devtool: 'source-map',
  plugins: [
    ...config.plugins,
    // new ClosureCompilerPlugin({
    //   compiler: {
    //     language_in: 'ECMASCRIPT6',
    //     language_out: 'ECMASCRIPT5',
    //     compilation_level: 'BASIC',
    //   },
    //   concurrency: 3,
    // }),
  ],
};
