"use strict";
var __spreadArray = (this && this.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.useAppStore = void 0;
var zustand_1 = require("zustand");
var logCounter = 0;
exports.useAppStore = (0, zustand_1.create)(function (set) { return ({
    currentPage: 'dashboard',
    setPage: function (page) { return set({ currentPage: page }); },
    servers: [],
    setServers: function (servers) { return set({ servers: servers }); },
    availableUpdates: [],
    setAvailableUpdates: function (updates) { return set({ availableUpdates: updates }); },
    isCheckingUpdates: false,
    setCheckingUpdates: function (checking) { return set({ isCheckingUpdates: checking }); },
    integrityReport: null,
    setIntegrityReport: function (report) { return set({ integrityReport: report }); },
    securityInfo: null,
    setSecurityInfo: function (info) { return set({ securityInfo: info }); },
    logs: [],
    addLog: function (level, message) {
        return set(function (state) { return ({
            logs: __spreadArray(__spreadArray([], state.logs, true), [
                { id: ++logCounter, timestamp: new Date(), level: level, message: message },
            ], false).slice(-500),
        }); });
    },
    clearLogs: function () { return set({ logs: [] }); },
}); });
