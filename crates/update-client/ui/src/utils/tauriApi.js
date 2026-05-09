"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.api = void 0;
var core_1 = require("@tauri-apps/api/core");
exports.api = {
    // Server management
    getServers: function () {
        return (0, core_1.invoke)('get_servers');
    },
    addServer: function (url) {
        return (0, core_1.invoke)('add_server', { url: url });
    },
    removeServer: function (publisherId) {
        return (0, core_1.invoke)('remove_server', { publisherId: publisherId });
    },
    // Updates
    checkUpdates: function (publisherId, packageName) {
        return (0, core_1.invoke)('check_updates', { publisherId: publisherId, packageName: packageName });
    },
    checkAllUpdates: function () {
        return (0, core_1.invoke)('check_all_updates');
    },
    verifyManifest: function (manifest) {
        return (0, core_1.invoke)('verify_manifest', { manifest: manifest });
    },
    // Security
    getIntegrityReport: function () {
        return (0, core_1.invoke)('get_integrity_report');
    },
    getSecurityInfo: function () {
        return (0, core_1.invoke)('get_security_info');
    },
};
