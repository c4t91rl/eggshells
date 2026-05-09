"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
var jsx_runtime_1 = require("react/jsx-runtime");
var react_1 = require("react");
var appStore_1 = require("../store/appStore");
var tauriApi_1 = require("../utils/tauriApi");
var hi2_1 = require("react-icons/hi2");
var framer_motion_1 = require("framer-motion");
var ServerManager = function () {
    var _a = (0, appStore_1.useAppStore)(), servers = _a.servers, setServers = _a.setServers, addLog = _a.addLog;
    var _b = (0, react_1.useState)(''), newServerUrl = _b[0], setNewServerUrl = _b[1];
    var _c = (0, react_1.useState)(false), isAdding = _c[0], setIsAdding = _c[1];
    var _d = (0, react_1.useState)(false), showAddForm = _d[0], setShowAddForm = _d[1];
    var _e = (0, react_1.useState)(null), selectedServer = _e[0], setSelectedServer = _e[1];
    var handleAddServer = function () { return __awaiter(void 0, void 0, void 0, function () {
        var server, updated, err_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    if (!newServerUrl.trim())
                        return [2 /*return*/];
                    setIsAdding(true);
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 4, 5, 6]);
                    return [4 /*yield*/, tauriApi_1.api.addServer(newServerUrl.trim())];
                case 2:
                    server = _a.sent();
                    return [4 /*yield*/, tauriApi_1.api.getServers()];
                case 3:
                    updated = _a.sent();
                    setServers(updated);
                    setNewServerUrl('');
                    setShowAddForm(false);
                    addLog('success', "Server added: ".concat(server.publisher.name, " (").concat(server.publisher.id, ")"));
                    return [3 /*break*/, 6];
                case 4:
                    err_1 = _a.sent();
                    addLog('error', "Failed to add server: ".concat(err_1));
                    return [3 /*break*/, 6];
                case 5:
                    setIsAdding(false);
                    return [7 /*endfinally*/];
                case 6: return [2 /*return*/];
            }
        });
    }); };
    var handleRemoveServer = function (publisherId) { return __awaiter(void 0, void 0, void 0, function () {
        var updated, err_2;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 3, , 4]);
                    return [4 /*yield*/, tauriApi_1.api.removeServer(publisherId)];
                case 1:
                    _a.sent();
                    return [4 /*yield*/, tauriApi_1.api.getServers()];
                case 2:
                    updated = _a.sent();
                    setServers(updated);
                    setSelectedServer(null);
                    addLog('info', "Server removed: ".concat(publisherId));
                    return [3 /*break*/, 4];
                case 3:
                    err_2 = _a.sent();
                    addLog('error', "Failed to remove server: ".concat(err_2));
                    return [3 /*break*/, 4];
                case 4: return [2 /*return*/];
            }
        });
    }); };
    var getTrustBadge = function (level) {
        switch (level) {
            case 'Pinned': return (0, jsx_runtime_1.jsx)("span", { className: "status-ok", children: "\uD83D\uDCCC Pinned" });
            case 'Verified': return (0, jsx_runtime_1.jsx)("span", { className: "status-ok", children: "\u2705 Verified" });
            case 'TrustOnFirstUse': return (0, jsx_runtime_1.jsx)("span", { className: "status-warning", children: "\uD83E\uDD1D TOFU" });
            case 'Untrusted': return (0, jsx_runtime_1.jsx)("span", { className: "status-error", children: "\u26A0\uFE0F Untrusted" });
        }
    };
    return ((0, jsx_runtime_1.jsxs)("div", { className: "space-y-6", children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-center justify-between", children: [(0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h2", { className: "text-2xl font-bold text-dark-50", children: "Update Servers" }), (0, jsx_runtime_1.jsx)("p", { className: "text-dark-400 text-sm mt-1", children: "Manage trusted publishers and their signing keys" })] }), (0, jsx_runtime_1.jsxs)("button", { onClick: function () { return setShowAddForm(!showAddForm); }, className: "btn-primary flex items-center gap-2", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlinePlusCircle, { size: 18 }), "Add Server"] })] }), (0, jsx_runtime_1.jsx)(framer_motion_1.AnimatePresence, { children: showAddForm && ((0, jsx_runtime_1.jsxs)(framer_motion_1.motion.div, { initial: { opacity: 0, height: 0 }, animate: { opacity: 1, height: 'auto' }, exit: { opacity: 0, height: 0 }, className: "glass-card p-6 overflow-hidden", children: [(0, jsx_runtime_1.jsx)("h3", { className: "font-semibold text-dark-100 mb-4", children: "Add Update Server" }), (0, jsx_runtime_1.jsxs)("div", { className: "flex gap-3", children: [(0, jsx_runtime_1.jsx)("input", { type: "text", value: newServerUrl, onChange: function (e) { return setNewServerUrl(e.target.value); }, placeholder: "https://update-server.example.com", className: "input-field flex-1", onKeyDown: function (e) { return e.key === 'Enter' && handleAddServer(); } }), (0, jsx_runtime_1.jsx)("button", { onClick: handleAddServer, disabled: isAdding || !newServerUrl.trim(), className: "btn-primary whitespace-nowrap", children: isAdding ? 'Discovering...' : 'Discover & Add' })] }), (0, jsx_runtime_1.jsx)("p", { className: "text-xs text-dark-500 mt-2", children: "The client will connect to the server, fetch its public keys, and register it using Trust On First Use (TOFU) model." })] })) }), (0, jsx_runtime_1.jsx)("div", { className: "grid grid-cols-1 lg:grid-cols-2 gap-4", children: servers.map(function (server, idx) { return ((0, jsx_runtime_1.jsxs)(framer_motion_1.motion.div, { initial: { opacity: 0, y: 20 }, animate: { opacity: 1, y: 0 }, transition: { delay: idx * 0.05 }, className: "glass-card-hover p-5 cursor-pointer ".concat((selectedServer === null || selectedServer === void 0 ? void 0 : selectedServer.publisher.id) === server.publisher.id
                        ? 'ring-2 ring-primary-500/50'
                        : ''), onClick: function () { return setSelectedServer(server); }, children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-start justify-between mb-4", children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-center gap-3", children: [(0, jsx_runtime_1.jsx)("div", { className: "w-10 h-10 rounded-xl bg-primary-500/10 flex items-center justify-center", children: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineServer, { size: 20, className: "text-primary-400" }) }), (0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h4", { className: "font-semibold text-dark-100", children: server.publisher.name }), (0, jsx_runtime_1.jsx)("p", { className: "text-xs text-dark-400", children: server.publisher.id })] })] }), getTrustBadge(server.trust_level)] }), (0, jsx_runtime_1.jsxs)("div", { className: "space-y-2 text-sm", children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-center gap-2 text-dark-400", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineGlobeAlt, { size: 14 }), (0, jsx_runtime_1.jsx)("span", { className: "font-mono text-xs truncate", children: server.url })] }), (0, jsx_runtime_1.jsxs)("div", { className: "flex items-center gap-2 text-dark-400", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineKey, { size: 14 }), (0, jsx_runtime_1.jsx)("span", { className: "font-mono text-xs", children: server.publisher.algorithm === 'HybridEd25519MlDsa65'
                                                ? '🔐 Hybrid (Ed25519 + ML-DSA-65)'
                                                : server.publisher.algorithm === 'MlDsa65'
                                                    ? '🛡️ ML-DSA-65 (Dilithium3)'
                                                    : '🔑 Ed25519' })] }), (0, jsx_runtime_1.jsxs)("div", { className: "flex items-center gap-2 text-dark-400", children: [(0, jsx_runtime_1.jsx)("span", { className: "text-xs", children: "Key ID: " }), (0, jsx_runtime_1.jsx)("code", { className: "text-xs bg-dark-800 px-1.5 py-0.5 rounded font-mono text-dark-300", children: server.publisher.key_id })] })] }), (0, jsx_runtime_1.jsxs)("div", { className: "flex items-center justify-between mt-4 pt-3 border-t border-dark-700/50", children: [(0, jsx_runtime_1.jsx)("span", { className: "text-xs text-dark-500", children: server.last_checked
                                        ? "Last checked: ".concat(new Date(server.last_checked).toLocaleString())
                                        : 'Never checked' }), (0, jsx_runtime_1.jsx)("button", { onClick: function (e) {
                                        e.stopPropagation();
                                        handleRemoveServer(server.publisher.id);
                                    }, className: "text-red-400 hover:text-red-300 p-1 rounded-lg hover:bg-red-500/10 transition-colors", children: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineTrash, { size: 16 }) })] })] }, server.publisher.id)); }) }), servers.length === 0 && ((0, jsx_runtime_1.jsxs)("div", { className: "glass-card p-12 text-center", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineServer, { size: 48, className: "text-dark-600 mx-auto mb-4" }), (0, jsx_runtime_1.jsx)("h3", { className: "text-lg font-semibold text-dark-300", children: "No Servers Registered" }), (0, jsx_runtime_1.jsx)("p", { className: "text-dark-500 text-sm mt-2", children: "Add an update server to start receiving secure software updates." })] })), (0, jsx_runtime_1.jsx)(framer_motion_1.AnimatePresence, { children: selectedServer && ((0, jsx_runtime_1.jsxs)(framer_motion_1.motion.div, { initial: { opacity: 0, y: 20 }, animate: { opacity: 1, y: 0 }, exit: { opacity: 0, y: 20 }, className: "glass-card p-6", children: [(0, jsx_runtime_1.jsxs)("h3", { className: "section-title mb-4", children: ["Server Details: ", selectedServer.publisher.name] }), (0, jsx_runtime_1.jsxs)("div", { className: "grid grid-cols-1 md:grid-cols-2 gap-6", children: [(0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h4", { className: "text-sm font-semibold text-dark-300 mb-3", children: "Public Keys" }), selectedServer.publisher.ed25519_public_key && ((0, jsx_runtime_1.jsxs)("div", { className: "mb-3", children: [(0, jsx_runtime_1.jsx)("label", { className: "text-xs text-dark-500 block mb-1", children: "Ed25519 Public Key" }), (0, jsx_runtime_1.jsx)("code", { className: "text-xs bg-dark-800 p-2 rounded-lg block font-mono text-dark-300 break-all", children: selectedServer.publisher.ed25519_public_key })] })), selectedServer.publisher.ml_dsa_public_key && ((0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("label", { className: "text-xs text-dark-500 block mb-1", children: "ML-DSA-65 Public Key" }), (0, jsx_runtime_1.jsxs)("code", { className: "text-xs bg-dark-800 p-2 rounded-lg block font-mono text-quantum-300 break-all max-h-20 overflow-y-auto", children: [selectedServer.publisher.ml_dsa_public_key.substring(0, 120), "..."] }), (0, jsx_runtime_1.jsxs)("span", { className: "text-xs text-dark-500", children: ["(", selectedServer.publisher.ml_dsa_public_key.length, " chars)"] })] }))] }), (0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h4", { className: "text-sm font-semibold text-dark-300 mb-3", children: "Trust Information" }), (0, jsx_runtime_1.jsxs)("div", { className: "space-y-3", children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex justify-between text-sm", children: [(0, jsx_runtime_1.jsx)("span", { className: "text-dark-400", children: "Trust Level" }), getTrustBadge(selectedServer.trust_level)] }), (0, jsx_runtime_1.jsxs)("div", { className: "flex justify-between text-sm", children: [(0, jsx_runtime_1.jsx)("span", { className: "text-dark-400", children: "Algorithm" }), (0, jsx_runtime_1.jsx)("span", { className: "text-dark-200 font-mono text-xs", children: selectedServer.publisher.algorithm })] }), (0, jsx_runtime_1.jsxs)("div", { className: "flex justify-between text-sm", children: [(0, jsx_runtime_1.jsx)("span", { className: "text-dark-400", children: "Registered" }), (0, jsx_runtime_1.jsx)("span", { className: "text-dark-200 text-xs", children: new Date(selectedServer.publisher.created_at).toLocaleDateString() })] })] })] })] })] })) })] }));
};
exports.default = ServerManager;
