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
            case 'Pinned': return <span className="status-ok">📌 Pinned</span>;
            case 'Verified': return <span className="status-ok">✅ Verified</span>;
            case 'TrustOnFirstUse': return <span className="status-warning">🤝 TOFU</span>;
            case 'Untrusted': return <span className="status-error">⚠️ Untrusted</span>;
        }
    };
    return (<div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Update Servers</h2>
          <p className="text-dark-400 text-sm mt-1">
            Manage trusted publishers and their signing keys
          </p>
        </div>
        <button onClick={function () { return setShowAddForm(!showAddForm); }} className="btn-primary flex items-center gap-2">
          <hi2_1.HiOutlinePlusCircle size={18}/>
          Add Server
        </button>
      </div>

      {/* Add Server Form */}
      <framer_motion_1.AnimatePresence>
        {showAddForm && (<framer_motion_1.motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }} exit={{ opacity: 0, height: 0 }} className="glass-card p-6 overflow-hidden">
            <h3 className="font-semibold text-dark-100 mb-4">Add Update Server</h3>
            <div className="flex gap-3">
              <input type="text" value={newServerUrl} onChange={function (e) { return setNewServerUrl(e.target.value); }} placeholder="https://update-server.example.com" className="input-field flex-1" onKeyDown={function (e) { return e.key === 'Enter' && handleAddServer(); }}/>
              <button onClick={handleAddServer} disabled={isAdding || !newServerUrl.trim()} className="btn-primary whitespace-nowrap">
                {isAdding ? 'Discovering...' : 'Discover & Add'}
              </button>
            </div>
            <p className="text-xs text-dark-500 mt-2">
              The client will connect to the server, fetch its public keys, and register it
              using Trust On First Use (TOFU) model.
            </p>
          </framer_motion_1.motion.div>)}
      </framer_motion_1.AnimatePresence>

      {/* Server List */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {servers.map(function (server, idx) { return (<framer_motion_1.motion.div key={server.publisher.id} initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: idx * 0.05 }} className={"glass-card-hover p-5 cursor-pointer ".concat((selectedServer === null || selectedServer === void 0 ? void 0 : selectedServer.publisher.id) === server.publisher.id
                ? 'ring-2 ring-primary-500/50'
                : '')} onClick={function () { return setSelectedServer(server); }}>
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl bg-primary-500/10 flex items-center justify-center">
                  <hi2_1.HiOutlineServer size={20} className="text-primary-400"/>
                </div>
                <div>
                  <h4 className="font-semibold text-dark-100">{server.publisher.name}</h4>
                  <p className="text-xs text-dark-400">{server.publisher.id}</p>
                </div>
              </div>
              {getTrustBadge(server.trust_level)}
            </div>

            <div className="space-y-2 text-sm">
              <div className="flex items-center gap-2 text-dark-400">
                <hi2_1.HiOutlineGlobeAlt size={14}/>
                <span className="font-mono text-xs truncate">{server.url}</span>
              </div>
              <div className="flex items-center gap-2 text-dark-400">
                <hi2_1.HiOutlineKey size={14}/>
                <span className="font-mono text-xs">
                  {server.publisher.algorithm === 'HybridEd25519MlDsa65'
                ? '🔐 Hybrid (Ed25519 + ML-DSA-65)'
                : server.publisher.algorithm === 'MlDsa65'
                    ? '🛡️ ML-DSA-65 (Dilithium3)'
                    : '🔑 Ed25519'}
                </span>
              </div>
              <div className="flex items-center gap-2 text-dark-400">
                <span className="text-xs">Key ID: </span>
                <code className="text-xs bg-dark-800 px-1.5 py-0.5 rounded font-mono text-dark-300">
                  {server.publisher.key_id}
                </code>
              </div>
            </div>

            <div className="flex items-center justify-between mt-4 pt-3 border-t border-dark-700/50">
              <span className="text-xs text-dark-500">
                {server.last_checked
                ? "Last checked: ".concat(new Date(server.last_checked).toLocaleString())
                : 'Never checked'}
              </span>
              <button onClick={function (e) {
                e.stopPropagation();
                handleRemoveServer(server.publisher.id);
            }} className="text-red-400 hover:text-red-300 p-1 rounded-lg hover:bg-red-500/10 transition-colors">
                <hi2_1.HiOutlineTrash size={16}/>
              </button>
            </div>
          </framer_motion_1.motion.div>); })}
      </div>

      {servers.length === 0 && (<div className="glass-card p-12 text-center">
          <hi2_1.HiOutlineServer size={48} className="text-dark-600 mx-auto mb-4"/>
          <h3 className="text-lg font-semibold text-dark-300">No Servers Registered</h3>
          <p className="text-dark-500 text-sm mt-2">
            Add an update server to start receiving secure software updates.
          </p>
        </div>)}

      {/* Server Detail Panel */}
      <framer_motion_1.AnimatePresence>
        {selectedServer && (<framer_motion_1.motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: 20 }} className="glass-card p-6">
            <h3 className="section-title mb-4">
              Server Details: {selectedServer.publisher.name}
            </h3>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Public Keys */}
              <div>
                <h4 className="text-sm font-semibold text-dark-300 mb-3">Public Keys</h4>
                {selectedServer.publisher.ed25519_public_key && (<div className="mb-3">
                    <label className="text-xs text-dark-500 block mb-1">Ed25519 Public Key</label>
                    <code className="text-xs bg-dark-800 p-2 rounded-lg block font-mono text-dark-300 break-all">
                      {selectedServer.publisher.ed25519_public_key}
                    </code>
                  </div>)}
                {selectedServer.publisher.ml_dsa_public_key && (<div>
                    <label className="text-xs text-dark-500 block mb-1">ML-DSA-65 Public Key</label>
                    <code className="text-xs bg-dark-800 p-2 rounded-lg block font-mono text-quantum-300 break-all max-h-20 overflow-y-auto">
                      {selectedServer.publisher.ml_dsa_public_key.substring(0, 120)}...
                    </code>
                    <span className="text-xs text-dark-500">
                      ({selectedServer.publisher.ml_dsa_public_key.length} chars)
                    </span>
                  </div>)}
              </div>

              {/* Trust Info */}
              <div>
                <h4 className="text-sm font-semibold text-dark-300 mb-3">Trust Information</h4>
                <div className="space-y-3">
                  <div className="flex justify-between text-sm">
                    <span className="text-dark-400">Trust Level</span>
                    {getTrustBadge(selectedServer.trust_level)}
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-dark-400">Algorithm</span>
                    <span className="text-dark-200 font-mono text-xs">
                      {selectedServer.publisher.algorithm}
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-dark-400">Registered</span>
                    <span className="text-dark-200 text-xs">
                      {new Date(selectedServer.publisher.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </framer_motion_1.motion.div>)}
      </framer_motion_1.AnimatePresence>
    </div>);
};
exports.default = ServerManager;
