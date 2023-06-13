const NodePolyfillPlugin = require('node-polyfill-webpack-plugin');

module.exports = function override (config, env) {
    console.log('override')

    config.plugins = [
		new NodePolyfillPlugin({
			excludeAliases: ['console']
		})
	].concat(config.plugins)
    
    return config
}