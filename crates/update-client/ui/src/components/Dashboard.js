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
var Dashboard = function () {
    var _a;
    var _b = (0, appStore_1.useAppStore)(), servers = _b.servers, availableUpdates = _b.availableUpdates, setAvailableUpdates = _b.setAvailableUpdates, isCheckingUpdates = _b.isCheckingUpdates, setCheckingUpdates = _b.setCheckingUpdates, integrityReport = _b.integrityReport, securityInfo = _b.securityInfo, addLog = _b.addLog;
    var handleCheckUpdates = function () { return __awaiter(void 0, void 0, void 0, function () {
        var updates, err_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    setCheckingUpdates(true);
                    addLog('info', 'Checking for updates across all servers...');
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 3, 4, 5]);
                    return [4 /*yield*/, tauriApi_1.api.checkAllUpdates()];
                case 2:
                    updates = _a.sent();
                    setAvailableUpdates(updates);
                    addLog('success', "Found ".concat(updates.length, " available update(s)"));
                    return [3 /*break*/, 5];
                case 3:
                    err_1 = _a.sent();
                    addLog('error', "Update check failed: ".concat(err_1));
                    return [3 /*break*/, 5];
                case 4:
                    setCheckingUpdates(false);
                    return [7 /*endfinally*/];
                case 5: return [2 /*return*/];
            }
        });
    }); };
    var statCards = [
        {
            title: 'Registered Servers',
            value: servers.length,
            icon: <hi2_1.HiOutlineServer size={24}/>,
            color: 'text-blue-400',
            bg: 'bg-blue-500/10',
        },
        {
            title: 'Available Updates',
            value: availableUpdates.length,
            icon: <hi2_1.HiOutlineArrowDown size={24}/>,
            color: 'text-primary-400',
            bg: 'bg-primary-500/10',
        },
        {
            title: 'Verified Signatures',
            value: availableUpdates.filter(function (u) { return u.verification.is_valid; }).length,
            icon: <hi2_1.HiOutlineCheckCircle size={24}/>,
            color: 'text-green-400',
            bg: 'bg-green-500/10',
        },
        {
            title: 'Security Warnings',
            value: ((_a = integrityReport === null || integrityReport === void 0 ? void 0 : integrityReport.checks.filter(function (c) { return c.status !== 'Ok'; }).length) !== null && _a !== void 0 ? _a : 0) +
                availableUpdates.filter(function (u) { return !u.verification.is_valid; }).length,
            icon: <hi2_1.HiOutlineExclamationTriangle size={24}/>,
            color: 'text-yellow-400',
            bg: 'bg-yellow-500/10',
        },
    ];
    return (<div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Dashboard</h2>
          <p className="text-dark-400 text-sm mt-1">
            Post-quantum secure software update management
          </p>
        </div>
        <button onClick={handleCheckUpdates} disabled={isCheckingUpdates} className="btn-primary flex items-center gap-2">
          <hi2_1.HiOutlineArrowPath size={18} className={isCheckingUpdates ? 'animate-spin' : ''}/>
          {isCheckingUpdates ? 'Checking...' : 'Check for Updates'}
        </button>
      </div>

      {/* Stat Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {statCards.map(function (stat, index) { return (<framer_motion_1.motion.div key={stat.title} initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: index * 0.1 }} className="glass-card p-5">
            <div className="flex items-center justify-between mb-3">
              <div className={"".concat(stat.bg, " p-2.5 rounded-xl ").concat(stat.color)}>
                {stat.icon}
              </div>
            </div>
            <div className="text-3xl font-bold text-dark-50">{stat.value}</div>
            <div className="text-sm text-dark-400 mt-1">{stat.title}</div>
          </framer_motion_1.motion.div>); })}
      </div>

      {/* Cryptography Info */}
      {securityInfo && (<framer_motion_1.motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.4 }} className="glass-card p-6">
          <h3 className="section-title flex items-center gap-2">
            <hi2_1.HiOutlineShieldCheck size={20} className="text-quantum-400"/>
            Supported Cryptography
          </h3>
          <p className="section-subtitle mb-4">
            Post-quantum and classical algorithms available for signature verification
          </p>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {securityInfo.supported_algorithms.map(function (algo) { return (<div key={algo.name} className="bg-dark-800/50 rounded-xl p-4 border border-dark-700/50">
                <div className="flex items-center justify-between mb-3">
                  <h4 className="font-semibold text-dark-100 text-sm">{algo.name}</h4>
                  {algo.quantum_safe ? (<span className="quantum-badge">
                      🛡️ Quantum Safe
                    </span>) : (<span className="status-info">Classical</span>)}
                </div>
                <div className="space-y-1.5 text-xs text-dark-400">
                  <div className="flex justify-between">
                    <span>Type:</span>
                    <span className="text-dark-300 font-mono">{algo.algorithm_type}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Key Size:</span>
                    <span className="text-dark-300 font-mono">{algo.key_size}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Security:</span>
                    <span className="text-dark-300 font-mono">{algo.security_level}</span>
                  </div>
                </div>
              </div>); })}
          </div>
        </framer_motion_1.motion.div>)}

      {/* Recent Updates */}
      {availableUpdates.length > 0 && (<framer_motion_1.motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.5 }} className="glass-card p-6">
          <h3 className="section-title">Recent Updates</h3>
          <p className="section-subtitle mb-4">Latest available updates from registered servers</p>

          <div className="space-y-3">
            {availableUpdates.slice(0, 5).map(function (update, idx) { return (<div key={idx} className="flex items-center justify-between bg-dark-800/50 rounded-xl p-4 border border-dark-700/50">
                <div className="flex items-center gap-4">
                  <div className={"w-10 h-10 rounded-xl flex items-center justify-center ".concat(update.verification.is_valid
                    ? 'bg-green-500/15 text-green-400'
                    : 'bg-red-500/15 text-red-400')}>
                    {update.verification.is_valid ? (<hi2_1.HiOutlineCheckCircle size={24}/>) : (<hi2_1.HiOutlineExclamationTriangle size={24}/>)}
                  </div>
                  <div>
                    <h4 className="font-semibold text-dark-100">
                      {update.manifest.manifest.package_name}
                    </h4>
                    <p className="text-xs text-dark-400">
                      v{update.manifest.manifest.version} • {update.publisher_name}
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  {update.manifest.signatures.map(function (sig, sigIdx) { return (<span key={sigIdx} className={sig.algorithm === 'HybridEd25519MlDsa65'
                        ? 'quantum-badge'
                        : sig.algorithm === 'MlDsa65'
                            ? 'quantum-badge'
                            : 'status-info'}>
                      {sig.algorithm === 'HybridEd25519MlDsa65'
                        ? '🔐 Hybrid PQ'
                        : sig.algorithm === 'MlDsa65'
                            ? '🛡️ ML-DSA'
                            : '🔑 Ed25519'}
                    </span>); })}
                  <button className="btn-primary text-sm py-1.5">Install</button>
                </div>
              </div>); })}
          </div>
        </framer_motion_1.motion.div>)}
    </div>);
};
exports.default = Dashboard;
