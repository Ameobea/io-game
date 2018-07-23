const config = require('./webpack.config');

module.exports = {
  ...config,
  // mode: 'production', // disabled due to WebPack Wasm parsing bug
  devtool: 'source-map',
};
